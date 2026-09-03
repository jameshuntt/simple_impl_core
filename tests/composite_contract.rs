use quote::ToTokens;
use simple_impl_core::{
    CompositeEntry,
    CompositeEntryContract,
    CompositeEntryKind,
    CompositeShellContract,
    CompositeSurfaceContract,
};
use syn::{parse_quote, DeriveInput, Fields};

fn field_entries(input: DeriveInput) -> Vec<CompositeEntry> {
    let fields = match input.data {
        syn::Data::Struct(data) => match data.fields {
            Fields::Named(named) => named.named,
            _ => panic!("expected named fields"),
        },
        _ => panic!("expected struct"),
    };

    fields
        .iter()
        .filter_map(|field| CompositeEntry::from_field(field).unwrap())
        .collect()
}

fn registry_entries(input: DeriveInput) -> Vec<CompositeEntry> {
    input
        .attrs
        .iter()
        .filter_map(|attr| CompositeEntry::from_registry_attr(attr).unwrap())
        .collect()
}

#[test]
fn registry_and_field_layouts_normalize_to_same_contract() {
    let registry: DeriveInput = parse_quote! {
        #[composite(command = "add", method = "add", ty = GitRemoteAdd, init = "name,url")]
        pub struct GitRemote;
    };

    let field_style: DeriveInput = parse_quote! {
        pub struct GitRemote {
            #[composite(command, init = "name,url")]
            add: GitRemoteAdd,
        }
    };

    let registry_contract = CompositeEntryContract::from_entry(&registry_entries(registry)[0]);
    let field_contract = CompositeEntryContract::from_entry(&field_entries(field_style)[0]);

    assert_eq!(registry_contract, field_contract);
    assert_eq!(registry_contract.method, "add");
    assert_eq!(registry_contract.segment, "add");
    assert_eq!(registry_contract.ty, "GitRemoteAdd");
    assert_eq!(registry_contract.init_args, vec!["name", "url"]);
}

#[test]
fn imperative_entries_can_form_the_same_contract_without_attrs_or_macros() {
    let manual = CompositeEntryContract::command("add", "add", "GitRemoteAdd")
        .with_init_args(["name", "url"]);

    let attr_input: DeriveInput = parse_quote! {
        #[composite(command = "add", ty = GitRemoteAdd, init = "name,url")]
        pub struct GitRemote;
    };

    let attr_entry = registry_entries(attr_input).remove(0);
    let attr_contract = CompositeEntryContract::from_entry(&attr_entry);

    assert_eq!(manual, attr_contract);
}

#[test]
fn shell_and_surface_contracts_validate_required_acknowledgments() {
    let shell = CompositeShellContract::new(
        "git",
        vec![CompositeEntryContract::command("commit", "commit", "GitCommit")],
    );

    let surface = CompositeSurfaceContract::new(
        "remote",
        vec![CompositeEntryContract::command("add", "add", "GitRemoteAdd")
            .with_init_args(["name", "url"])],
    );

    assert!(shell.validate().is_ok());
    assert!(surface.validate().is_ok());
}

#[test]
fn contract_validation_rejects_missing_or_duplicate_acknowledgments() {
    let missing_ty = CompositeEntryContract::command("add", "add", "");
    assert!(missing_ty.validate().unwrap_err().contains("non-empty ty"));

    let duplicate_init = CompositeEntryContract::command("add", "add", "GitRemoteAdd")
        .with_init_args(["name", "name"]);
    assert!(
        duplicate_init
            .validate()
            .unwrap_err()
            .contains("duplicate init arg `name`")
    );

    let duplicate_method = CompositeSurfaceContract::new(
        "remote",
        vec![
            CompositeEntryContract::command("add", "add", "GitRemoteAdd"),
            CompositeEntryContract::command("add", "remove", "GitRemoteRemove"),
        ],
    );
    assert!(
        duplicate_method
            .validate()
            .unwrap_err()
            .contains("duplicate composite method `add`")
    );
}

#[test]
fn contract_keeps_raw_semantic_acknowledgments_visible() {
    let input: DeriveInput = parse_quote! {
        #[composite(surface = "remote", ty = GitRemote)]
        pub struct Git;
    };

    let entry = registry_entries(input).remove(0);
    let contract = CompositeEntryContract::from_entry(&entry);

    assert_eq!(contract.kind, CompositeEntryKind::Surface);
    assert!(contract.is_surface());
    assert!(!contract.is_command());
    assert_eq!(entry.ty.to_token_stream().to_string(), contract.ty);
}
