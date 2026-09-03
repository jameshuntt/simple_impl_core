use simple_impl_attr_kit::{parse_field_validation_attrs, parse_validation_attrs};
use simple_impl_core::{ValidationRule, ValidationSpec};
use syn::{parse_quote, Data, DeriveInput, Field, Fields};

fn first_named_field(input: DeriveInput) -> Field {
    let Data::Struct(data) = input.data else {
        panic!("expected struct");
    };
    let Fields::Named(fields) = data.fields else {
        panic!("expected named fields");
    };
    fields.named.into_iter().next().expect("missing field")
}

#[test]
fn lowers_field_validation_attrs_into_core_rules() {
    let input: DeriveInput = parse_quote! {
        pub struct Example {
            #[invalid_without("force")]
            #[conflicts_with("dry_run")]
            delete: bool,
        }
    };
    let field = first_named_field(input);

    let parsed = parse_field_validation_attrs("delete", &field.attrs).unwrap();
    let spec = ValidationSpec::from(parsed);

    assert_eq!(
        spec.rules(),
        &[
            ValidationRule::InvalidWithout {
                field: "delete".into(),
                required: "force".into(),
            },
            ValidationRule::ConflictsWith {
                field: "delete".into(),
                conflicts_with: "dry_run".into(),
            },
        ]
    );
}

#[test]
fn lowers_validate_with_attrs_into_custom_function_rules() {
    let input: DeriveInput = parse_quote! {
        pub struct Example {
            #[validate(with = "validate_cp_preflight")]
            _validate: (),
        }
    };
    let field = first_named_field(input);

    let parsed = parse_field_validation_attrs("_validate", &field.attrs).unwrap();
    let spec = ValidationSpec::from(parsed);

    assert_eq!(
        spec.rules(),
        &[ValidationRule::CustomFunction {
            function_path: "validate_cp_preflight".into(),
        }]
    );
}

#[test]
fn lowers_set_validation_attrs_into_core_rules() {
    let input: DeriveInput = parse_quote! {
        #[one_of("json", "yaml")]
        #[at_least_one_of("source", "destination")]
        pub struct Example;
    };

    let parsed = parse_validation_attrs(&input.attrs).unwrap();
    let spec = ValidationSpec::from(parsed);

    assert_eq!(
        spec.rules(),
        &[
            ValidationRule::OneOf {
                fields: vec!["json".into(), "yaml".into()],
            },
            ValidationRule::AtLeastOneOf {
                fields: vec!["source".into(), "destination".into()],
            },
        ]
    );
}
