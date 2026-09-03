// --------------------
// type helpers
// --------------------

use syn::{
    Type,
};

pub fn type_is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "bool"))
}

pub fn type_is_string(ty: &Type) -> bool {
    matches!(ty, Type::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "String"))
}

pub fn type_option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(p) = ty else { return None; };
    let seg = p.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else { return None; };
    let first = ab.args.first()?;
    match first {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    }
}

pub fn type_vec_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(p) = ty else { return None; };
    let seg = p.path.segments.last()?;
    if seg.ident != "Vec" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else { return None; };
    let first = ab.args.first()?;
    match first {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    }
}

pub fn type_option_vec_inner(ty: &Type) -> Option<&Type> {
    type_option_inner(ty).and_then(type_vec_inner)
}

pub fn type_is_uint(ty: &Type) -> bool {
    matches!(ty,
        Type::Path(p) if p.path.segments.last().is_some_and(|s| matches!(
            s.ident.to_string().as_str(),
            "u8" | "u16" | "u32" | "u64" | "usize"
        ))
    )
}
