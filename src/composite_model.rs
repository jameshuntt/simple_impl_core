//! Parse-side model for future composite command derives.
//!
//! This file intentionally does not generate code yet. It defines the contract
//! that both composite syntaxes must normalize into before codegen.

use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
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
        let Some(field_ident) = &field.ident else {
            return Ok(None);
        };

        for attr in &field.attrs {
            if !attr.path().is_ident("composite") {
                continue;
            }

            let bag = parse_field_composite_attr(attr)?;
            let (kind, segment) = parse_composite_kind(&bag, attr, true)?;
            let segment = segment.unwrap_or_else(|| field_ident.to_string());
            let method = parse_optional_method(&bag, attr)?.unwrap_or_else(|| field_ident.clone());

            let init_args = parse_init_args(&bag, attr)?;

            return Ok(Some(Self {
                kind,
                method,
                segment,
                ty: field.ty.clone(),
                init_args,
                field: Some(field_ident.clone()),
            }));
        }

        Ok(None)
    }

    /// Parse registry-style declarations:
    ///
    /// ```ignore
    /// #[composite(command = "add", method = "add", ty = GitRemoteAdd)]
    /// pub struct GitRemote;
    /// ```
    pub fn from_registry_attr(attr: &Attribute) -> syn::Result<Option<Self>> {
        if !attr.path().is_ident("composite") {
            return Ok(None);
        }

        let bag = parse_registry_composite_attr(attr)?;
        let (kind, segment) = parse_composite_kind(&bag, attr, false)?;
        let segment = segment.ok_or_else(|| {
            syn::Error::new_spanned(attr, "registry-style #[composite] requires command/surface value")
        })?;

        let method = match parse_optional_method(&bag, attr)? {
            Some(method) => method,
            None => syn::parse_str::<Ident>(&segment.replace('-', "_"))?,
        };

        let ty_tokens = bag.optional_tokens("ty")?.ok_or_else(|| {
            syn::Error::new_spanned(attr, "registry-style #[composite] requires ty = SomeType")
        })?;
        let ty = syn::parse_str::<Type>(ty_tokens)?;

        let init_args = parse_init_args(&bag, attr)?;

        Ok(Some(Self {
            kind,
            method,
            segment,
            ty,
            init_args,
            field: None,
        }))
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
        .map(syn::parse_str::<Ident>)
        .transpose()
        .map_err(|err| syn::Error::new_spanned(attr, err.to_string()))
}


fn parse_init_args(bag: &AttrBag, attr: &Attribute) -> syn::Result<Vec<Ident>> {
    let Some(raw) = bag.optional_str("init")? else {
        return Ok(Vec::new());
    };

    raw.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            syn::parse_str::<Ident>(part)
                .map_err(|err| syn::Error::new_spanned(attr, err.to_string()))
        })
        .collect()
}
