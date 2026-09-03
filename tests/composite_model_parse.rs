use quote::ToTokens;
use simple_impl_core::{CompositeEntry, CompositeEntryKind};
use syn::{parse_quote, DeriveInput, Fields};

#[test]
fn parses_field_style_composite_entries() {
    let input: DeriveInput = parse_quote! {
        pub struct GitRemote {
            #[composite(command)]
            add: GitRemoteAdd,

            #[composite(command = "set-url", method = "set_url")]
            set_url: GitRemoteSetUrl,

            #[composite(surface)]
            nested: GitRemoteNested,
        }
    };

    let fields = match input.data {
        syn::Data::Struct(data) => match data.fields {
            Fields::Named(named) => named.named,
            _ => panic!("expected named fields"),
        },
        _ => panic!("expected struct"),
    };

    let entries: Vec<_> = fields
        .iter()
        .filter_map(|field| CompositeEntry::from_field(field).unwrap())
        .collect();

    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].kind, CompositeEntryKind::Command);
    assert_eq!(entries[0].method.to_string(), "add");
    assert_eq!(entries[0].segment, "add");
    assert_eq!(entries[0].ty.to_token_stream().to_string(), "GitRemoteAdd");

    assert_eq!(entries[1].kind, CompositeEntryKind::Command);
    assert_eq!(entries[1].method.to_string(), "set_url");
    assert_eq!(entries[1].segment, "set-url");
    assert_eq!(entries[1].ty.to_token_stream().to_string(), "GitRemoteSetUrl");

    assert_eq!(entries[2].kind, CompositeEntryKind::Surface);
    assert_eq!(entries[2].method.to_string(), "nested");
    assert_eq!(entries[2].segment, "nested");
    assert_eq!(entries[2].ty.to_token_stream().to_string(), "GitRemoteNested");
}

#[test]
fn parses_registry_style_composite_entries() {
    let input: DeriveInput = parse_quote! {
        #[composite(command = "add", method = "add", ty = GitRemoteAdd)]
        #[composite(command = "set-url", method = "set_url", ty = GitRemoteSetUrl)]
        #[composite(surface = "nested", method = "nested", ty = GitRemoteNested)]
        pub struct GitRemote;
    };

    let entries: Vec<_> = input
        .attrs
        .iter()
        .filter_map(|attr| CompositeEntry::from_registry_attr(attr).unwrap())
        .collect();

    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].kind, CompositeEntryKind::Command);
    assert_eq!(entries[0].method.to_string(), "add");
    assert_eq!(entries[0].segment, "add");
    assert_eq!(entries[0].ty.to_token_stream().to_string(), "GitRemoteAdd");

    assert_eq!(entries[1].kind, CompositeEntryKind::Command);
    assert_eq!(entries[1].method.to_string(), "set_url");
    assert_eq!(entries[1].segment, "set-url");
    assert_eq!(entries[1].ty.to_token_stream().to_string(), "GitRemoteSetUrl");

    assert_eq!(entries[2].kind, CompositeEntryKind::Surface);
    assert_eq!(entries[2].method.to_string(), "nested");
    assert_eq!(entries[2].segment, "nested");
    assert_eq!(entries[2].ty.to_token_stream().to_string(), "GitRemoteNested");
}

#[test]
fn registry_style_infers_method_name_from_segment_when_missing() {
    let input: DeriveInput = parse_quote! {
        #[composite(command = "set-url", ty = GitRemoteSetUrl)]
        pub struct GitRemote;
    };

    let entry = CompositeEntry::from_registry_attr(&input.attrs[0])
        .unwrap()
        .unwrap();

    assert_eq!(entry.kind, CompositeEntryKind::Command);
    assert_eq!(entry.method.to_string(), "set_url");
    assert_eq!(entry.segment, "set-url");
    assert_eq!(entry.ty.to_token_stream().to_string(), "GitRemoteSetUrl");
}

#[test]
fn composite_parser_rejects_unknown_keys_through_attr_kit_schema() {
    let input: DeriveInput = parse_quote! {
        pub struct GitRemote {
            #[composite(command, unknown = "nope")]
            add: GitRemoteAdd,
        }
    };

    let fields = match input.data {
        syn::Data::Struct(data) => match data.fields {
            Fields::Named(named) => named.named,
            _ => panic!("expected named fields"),
        },
        _ => panic!("expected struct"),
    };

    let err = CompositeEntry::from_field(fields.iter().next().unwrap()).unwrap_err();
    assert!(err.to_string().contains("unknown attribute key `unknown`"));
}

#[test]
fn registry_style_rejects_flag_segment_without_explicit_value() {
    let input: DeriveInput = parse_quote! {
        #[composite(command, ty = GitRemoteAdd)]
        pub struct GitRemote;
    };

    let err = CompositeEntry::from_registry_attr(&input.attrs[0]).unwrap_err();
    assert!(
        err.to_string()
            .contains("registry-style #[composite] requires `command = \"segment\"`")
    );
}

#[test]
fn composite_parser_rejects_command_and_surface_together() {
    let input: DeriveInput = parse_quote! {
        pub struct GitRemote {
            #[composite(command, surface)]
            add: GitRemoteAdd,
        }
    };

    let fields = match input.data {
        syn::Data::Struct(data) => match data.fields {
            Fields::Named(named) => named.named,
            _ => panic!("expected named fields"),
        },
        _ => panic!("expected struct"),
    };

    let err = CompositeEntry::from_field(fields.iter().next().unwrap()).unwrap_err();
    assert!(
        err.to_string()
            .contains("cannot contain both `command` and `surface`")
    );
}

#[test]
fn registry_style_parses_init_args_into_core_entry() {
    let input: DeriveInput = parse_quote! {
        #[composite(command = "add", ty = GitRemoteAdd, init = "name,url")]
        pub struct GitRemote;
    };

    let entry = CompositeEntry::from_registry_attr(&input.attrs[0])
        .unwrap()
        .unwrap();

    assert_eq!(entry.kind, CompositeEntryKind::Command);
    assert_eq!(entry.method.to_string(), "add");
    assert_eq!(entry.segment, "add");
    assert_eq!(entry.ty.to_token_stream().to_string(), "GitRemoteAdd");
    assert_eq!(
        entry
            .init_args
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["name", "url"]
    );
}

#[test]
fn field_style_parses_init_args_into_core_entry() {
    let input: DeriveInput = parse_quote! {
        pub struct GitRemote {
            #[composite(command, init = "name,url")]
            add: GitRemoteAdd,
        }
    };

    let fields = match input.data {
        syn::Data::Struct(data) => match data.fields {
            Fields::Named(named) => named.named,
            _ => panic!("expected named fields"),
        },
        _ => panic!("expected struct"),
    };

    let entry = CompositeEntry::from_field(fields.iter().next().unwrap())
        .unwrap()
        .unwrap();

    assert_eq!(entry.kind, CompositeEntryKind::Command);
    assert_eq!(entry.method.to_string(), "add");
    assert_eq!(entry.segment, "add");
    assert_eq!(entry.ty.to_token_stream().to_string(), "GitRemoteAdd");
    assert_eq!(
        entry
            .init_args
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["name", "url"]
    );
}
