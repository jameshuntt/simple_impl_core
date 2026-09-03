use simple_impl_core::{ValidationRule, ValidationSpec};

#[test]
fn validation_rule_constructors_model_dependency_rules() {
    let rules = vec![
        ValidationRule::invalid_without("delete", "force"),
        ValidationRule::only_pair_with("force", "delete"),
        ValidationRule::conflicts_with("json", "yaml"),
        ValidationRule::requires("recursive", "source"),
    ];

    assert_eq!(
        rules,
        vec![
            ValidationRule::InvalidWithout {
                field: "delete".into(),
                required: "force".into(),
            },
            ValidationRule::OnlyPairWith {
                field: "force".into(),
                paired_with: "delete".into(),
            },
            ValidationRule::ConflictsWith {
                field: "json".into(),
                conflicts_with: "yaml".into(),
            },
            ValidationRule::Requires {
                field: "recursive".into(),
                required: "source".into(),
            },
        ]
    );

    assert!(rules.iter().all(ValidationRule::is_structural));
}

#[test]
fn validation_rule_constructors_model_set_rules_and_custom_hooks() {
    let exact = ValidationRule::one_of(["json", "yaml", "toml"]);
    let at_least = ValidationRule::at_least_one_of(["source", "destination"]);
    let custom = ValidationRule::custom_function("validate_cp_preflight");

    assert_eq!(
        exact,
        ValidationRule::OneOf {
            fields: vec!["json".into(), "yaml".into(), "toml".into()],
        }
    );
    assert_eq!(
        at_least,
        ValidationRule::AtLeastOneOf {
            fields: vec!["source".into(), "destination".into()],
        }
    );
    assert!(custom.is_custom_function());
    assert!(!custom.is_structural());
}

#[test]
fn validation_spec_collects_rules_in_order() {
    let mut spec = ValidationSpec::new();
    assert!(spec.is_empty());

    spec.invalid_without("delete", "force")
        .only_pair_with("force", "delete")
        .conflicts_with("json", "yaml")
        .one_of(["json", "yaml"])
        .at_least_one_of(["source", "destination"])
        .custom_function("validate_cp_preflight");

    assert_eq!(spec.rules().len(), 6);
    assert_eq!(
        spec.rules()[0],
        ValidationRule::InvalidWithout {
            field: "delete".into(),
            required: "force".into(),
        }
    );
    assert_eq!(
        spec.rules()[5],
        ValidationRule::CustomFunction {
            function_path: "validate_cp_preflight".into(),
        }
    );
}
