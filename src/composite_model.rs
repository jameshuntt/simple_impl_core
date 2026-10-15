//! Parse-side model for future composite command derives.
//!
//! This file intentionally does not generate code yet. It defines the contract
//! that both composite syntaxes must normalize into before codegen.

use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use liaise_proc_macro::{collect_all, SynBuffer};
use simple_impl_attr_kit::{AttrBag, AttrExpected, AttrSchema, AttrValue};
use syn::{Attribute, Field, Ident, Path, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeEntryKind {
    Command,
    Surface,
}

#[derive(Debug, Clone)]
pub struct CompositeEntry {
    pub kind: CompositeEntryKind,
    pub method: Ident,
    pub segment: String,
    pub ty: Type,
    pub init_args: Vec<Ident>,
    /// The declaring field, for field-style entries; the generated code reads
    /// it once so the field is not reported as dead.
    pub field: Option<Ident>,
}

#[derive(Debug, Clone)]
pub struct CompositeRootSpec {
    pub program: String,
    pub trait_path: Option<Path>,
    pub entries: Vec<CompositeEntry>,
}

impl CompositeEntry {
    /// Parse field-style declarations:
    ///
    /// ```ignore
    /// pub struct GitRemote {
    ///     #[composite(command)]
    ///     add: GitRemoteAdd,
    ///
    ///     #[composite(command = "set-url", method = "set_url")]
    ///     set_url: GitRemoteSetUrl,
    /// }
    /// ```
    pub fn from_field(field: &Field) -> syn::Result<Option<Self>> {
        let mut errs = SynBuffer::new();
        let entry = Self::from_field_into(field, &mut errs);
        errs.finish_with(entry)
    }

    /// [`Self::from_field`] for a caller that collects: every problem with
    /// the entry is reported into `errs`. The entry comes back whenever its
    /// kind could be read, with the field's own name for a method that could
    /// not, so the caller can still check the rest; it must not generate code
    /// once anything was reported.
    pub fn from_field_into(field: &Field, errs: &mut SynBuffer) -> Option<Self> {
        let field_ident = field.ident.as_ref()?;
        let attr = field.attrs.iter().find(|attr| attr.path().is_ident("composite"))?;

        let bag = errs.absorb(parse_field_composite_attr(attr))?;
        let kind = errs.absorb(parse_composite_kind(&bag, attr, true));
        let method = errs.absorb(parse_optional_method(&bag, attr)).flatten();
        let init_args = parse_init_args_into(&bag, attr, errs);
        let (kind, segment) = kind?;

        Some(Self {
            kind,
            method: method.unwrap_or_else(|| field_ident.clone()),
            segment: segment.unwrap_or_else(|| field_ident.to_string()),
            ty: field.ty.clone(),
            init_args,
            field: Some(field_ident.clone()),
        })
    }

    /// Parse registry-style declarations:
    ///
    /// ```ignore
    /// #[composite(command = "add", method = "add", ty = GitRemoteAdd)]
    /// pub struct GitRemote;
    /// ```
    pub fn from_registry_attr(attr: &Attribute) -> syn::Result<Option<Self>> {
        let mut errs = SynBuffer::new();
        let entry = Self::from_registry_attr_into(attr, &mut errs);
        errs.finish_with(entry)
    }

    /// [`Self::from_registry_attr`] for a caller that collects: every problem
    /// with the entry is reported into `errs`. The entry comes back whenever
    /// its kind could be read, with stand-ins for a method or type that could
    /// not, so the caller can still check the rest; it must not generate code
    /// once anything was reported.
    pub fn from_registry_attr_into(attr: &Attribute, errs: &mut SynBuffer) -> Option<Self> {
        if !attr.path().is_ident("composite") {
            return None;
        }

        let bag = errs.absorb(parse_registry_composite_attr(attr))?;
        let kind = errs.absorb(parse_composite_kind(&bag, attr, false));
        let segment = match &kind {
            Some((_, Some(segment))) => Some(segment.clone()),
            Some((_, None)) => {
                errs.push(syn::Error::new_spanned(attr, "registry-style #[composite] requires command/surface value"));
                None
            }
            None => None,
        };

        let method = match errs.absorb(parse_optional_method(&bag, attr)) {
            Some(Some(method)) => Some(method),
            Some(None) => segment.as_ref().and_then(|segment| {
                errs.absorb(syn::parse_str::<Ident>(&segment.replace('-', "_")).map_err(|e| {
                    syn::Error::new_spanned(attr, format!("`{segment}` does not make a method name; give method = \"...\": {e}"))
                }))
            }),
            None => None,
        };

        let ty = match errs.absorb(bag.optional_tokens("ty")) {
            Some(Some(ty_tokens)) => errs.absorb(
                syn::parse_str::<Type>(ty_tokens)
                    .map_err(|e| syn::Error::new_spanned(attr, format!("`ty = {ty_tokens}` is not a type: {e}"))),
            ),
            Some(None) => {
                errs.push(syn::Error::new_spanned(attr, "registry-style #[composite] requires ty = SomeType"));
                None
            }
            None => None,
        };

        let init_args = parse_init_args_into(&bag, attr, errs);
        let (kind, _) = kind?;

        Some(Self {
            kind,
            method: method.unwrap_or_else(|| Ident::new("__simple_impl_unnamed_entry", proc_macro2::Span::call_site())),
            segment: segment.unwrap_or_default(),
            ty: ty.unwrap_or_else(|| syn::parse_quote!(())),
            init_args,
            field: None,
        })
    }

    pub fn ty_tokens(&self) -> TokenStream2 {
        self.ty.to_token_stream()
    }
}

fn parse_field_composite_attr(attr: &Attribute) -> syn::Result<AttrBag> {
    let parsed = AttrBag::from_attr(attr, "composite")?;

    AttrSchema::new()
        .optional("command", AttrExpected::Any)
        .optional("surface", AttrExpected::Any)
        .optional("method", AttrExpected::String)
        .optional("init", AttrExpected::String)
        .validate(&parsed)
}

fn parse_registry_composite_attr(attr: &Attribute) -> syn::Result<AttrBag> {
    let parsed = AttrBag::from_attr(attr, "composite")?;

    AttrSchema::new()
        .optional("command", AttrExpected::Any)
        .optional("surface", AttrExpected::Any)
        .optional("method", AttrExpected::String)
        .optional("ty", AttrExpected::Tokens)
        .optional("init", AttrExpected::String)
        .validate(&parsed)
}

fn parse_composite_kind(
    bag: &AttrBag,
    attr: &Attribute,
    allow_flag_segment: bool,
) -> syn::Result<(CompositeEntryKind, Option<String>)> {
    let command = bag.get("command");
    let surface = bag.get("surface");

    match (command, surface) {
        (Some(_), Some(_)) => Err(syn::Error::new_spanned(
            attr,
            "#[composite(...)] cannot contain both `command` and `surface`",
        )),
        (Some(value), None) => Ok((
            CompositeEntryKind::Command,
            parse_segment_value(value, attr, "command", allow_flag_segment)?,
        )),
        (None, Some(value)) => Ok((
            CompositeEntryKind::Surface,
            parse_segment_value(value, attr, "surface", allow_flag_segment)?,
        )),
        (None, None) => Err(syn::Error::new_spanned(
            attr,
            "expected #[composite(command)] or #[composite(surface)]",
        )),
    }
}

fn parse_segment_value(
    value: &AttrValue,
    attr: &Attribute,
    key: &str,
    allow_flag_segment: bool,
) -> syn::Result<Option<String>> {
    match value {
        AttrValue::Str(value) => Ok(Some(value.clone())),
        AttrValue::Bool(true) if allow_flag_segment => Ok(None),
        AttrValue::Bool(true) => Err(syn::Error::new_spanned(
            attr,
            format!("registry-style #[composite] requires `{key} = \"segment\"`"),
        )),
        AttrValue::Bool(false) => Err(syn::Error::new_spanned(
            attr,
            format!("`{key}` cannot be false in #[composite(...)]"),
        )),
        other => Err(syn::Error::new_spanned(
            attr,
            format!(
                "expected string{} for `{key}`, got {}",
                if allow_flag_segment { " or flag" } else { "" },
                other.kind_name()
            ),
        )),
    }
}

fn parse_optional_method(bag: &AttrBag, attr: &Attribute) -> syn::Result<Option<Ident>> {
    bag.optional_str("method")?
        .map(|m| {
            syn::parse_str::<Ident>(m)
                .map_err(|err| syn::Error::new_spanned(attr, format!("method `{m}` is not an identifier: {err}")))
        })
        .transpose()
}


/// Every name in `init = "a, b"`: each bad one is reported, by name.
fn parse_init_args(bag: &AttrBag, attr: &Attribute) -> syn::Result<Vec<Ident>> {
    let Some(raw) = bag.optional_str("init")? else {
        return Ok(Vec::new());
    };

    collect_all(raw.split(',').map(str::trim).filter(|part| !part.is_empty()).map(|part| {
        syn::parse_str::<Ident>(part)
            .map_err(|err| syn::Error::new_spanned(attr, format!("init name `{part}` is not an identifier: {err}")))
    }))
}

/// [`parse_init_args`] for a caller that collects: written but unreadable
/// names become stand-ins, so "init was written" still holds for the checks
/// that only need that; nothing is generated once anything was reported.
fn parse_init_args_into(bag: &AttrBag, attr: &Attribute, errs: &mut SynBuffer) -> Vec<Ident> {
    match errs.absorb(parse_init_args(bag, attr)) {
        Some(names) => names,
        None => vec![Ident::new("__simple_impl_unnamed_init", proc_macro2::Span::call_site())],
    }
}
