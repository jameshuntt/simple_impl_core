//! Semantic validation model for command fields and command clauses.
//!
//! This module is intentionally pure data. It does not inspect the filesystem,
//! run commands, or generate tokens. It gives later passes a stable language for
//! structural command validation such as `invalid_without`, `only_pair_with`,
//! `conflicts_with`, and custom preflight hooks.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationRule {
    /// `field` is only valid when `required` is also active/present.
    ///
    /// This is the general semantic form behind dependency rules.
    Requires { field: String, required: String },

    /// `field` is invalid unless `required` is also active/present.
    ///
    /// This is intentionally kept as a separate readable variant even though it
    /// can lower to `Requires` later. It reads better at the call site for
    /// dangerous actions: `delete` is invalid without `force`.
    InvalidWithout { field: String, required: String },

    /// `field` cannot appear alone; it must be paired with `paired_with`.
    ///
    /// Example: `force` is only meaningful when paired with `delete`.
    OnlyPairWith { field: String, paired_with: String },

    /// `field` cannot be active/present with `conflicts_with`.
    ConflictsWith { field: String, conflicts_with: String },

    /// Exactly one field from the set must be active/present.
    OneOf { fields: Vec<String> },

    /// At least one field from the set must be active/present.
    AtLeastOneOf { fields: Vec<String> },

    /// Runtime/preflight validation hook. This is a semantic reference only;
    /// later derive/codegen passes decide how to call the function.
    CustomFunction { function_path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationSpec {
    rules: Vec<ValidationRule>,
}

impl ValidationSpec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_rules(rules: impl IntoIterator<Item = ValidationRule>) -> Self {
        Self {
            rules: rules.into_iter().collect(),
        }
    }

    pub fn rules(&self) -> &[ValidationRule] {
        &self.rules
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn push(&mut self, rule: ValidationRule) -> &mut Self {
        self.rules.push(rule);
        self
    }

    pub fn requires(
        &mut self,
        field: impl Into<String>,
        required: impl Into<String>,
    ) -> &mut Self {
        self.push(ValidationRule::requires(field, required))
    }

    pub fn invalid_without(
        &mut self,
        field: impl Into<String>,
        required: impl Into<String>,
    ) -> &mut Self {
        self.push(ValidationRule::invalid_without(field, required))
    }

    pub fn only_pair_with(
        &mut self,
        field: impl Into<String>,
        paired_with: impl Into<String>,
    ) -> &mut Self {
        self.push(ValidationRule::only_pair_with(field, paired_with))
    }

    pub fn conflicts_with(
        &mut self,
        field: impl Into<String>,
        conflicts_with: impl Into<String>,
    ) -> &mut Self {
        self.push(ValidationRule::conflicts_with(field, conflicts_with))
    }

    pub fn one_of(&mut self, fields: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        self.push(ValidationRule::one_of(fields))
    }

    pub fn at_least_one_of(
        &mut self,
        fields: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        self.push(ValidationRule::at_least_one_of(fields))
    }

    pub fn custom_function(&mut self, function_path: impl Into<String>) -> &mut Self {
        self.push(ValidationRule::custom_function(function_path))
    }
}

impl ValidationRule {
    pub fn requires(field: impl Into<String>, required: impl Into<String>) -> Self {
        Self::Requires {
            field: field.into(),
            required: required.into(),
        }
    }

    pub fn invalid_without(field: impl Into<String>, required: impl Into<String>) -> Self {
        Self::InvalidWithout {
            field: field.into(),
            required: required.into(),
        }
    }

    pub fn only_pair_with(field: impl Into<String>, paired_with: impl Into<String>) -> Self {
        Self::OnlyPairWith {
            field: field.into(),
            paired_with: paired_with.into(),
        }
    }

    pub fn conflicts_with(field: impl Into<String>, conflicts_with: impl Into<String>) -> Self {
        Self::ConflictsWith {
            field: field.into(),
            conflicts_with: conflicts_with.into(),
        }
    }

    pub fn one_of(fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::OneOf {
            fields: fields.into_iter().map(Into::into).collect(),
        }
    }

    pub fn at_least_one_of(fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::AtLeastOneOf {
            fields: fields.into_iter().map(Into::into).collect(),
        }
    }

    pub fn custom_function(function_path: impl Into<String>) -> Self {
        Self::CustomFunction {
            function_path: function_path.into(),
        }
    }

    pub fn is_structural(&self) -> bool {
        !matches!(self, Self::CustomFunction { .. })
    }

    pub fn is_custom_function(&self) -> bool {
        matches!(self, Self::CustomFunction { .. })
    }
}

impl From<simple_impl_attr_kit::ParsedValidationRule> for ValidationRule {
    fn from(rule: simple_impl_attr_kit::ParsedValidationRule) -> Self {
        match rule {
            simple_impl_attr_kit::ParsedValidationRule::Requires { field, required } => {
                Self::Requires { field, required }
            }
            simple_impl_attr_kit::ParsedValidationRule::InvalidWithout { field, required } => {
                Self::InvalidWithout { field, required }
            }
            simple_impl_attr_kit::ParsedValidationRule::OnlyPairWith {
                field,
                paired_with,
            } => Self::OnlyPairWith { field, paired_with },
            simple_impl_attr_kit::ParsedValidationRule::ConflictsWith {
                field,
                conflicts_with,
            } => Self::ConflictsWith {
                field,
                conflicts_with,
            },
            simple_impl_attr_kit::ParsedValidationRule::OneOf { fields } => Self::OneOf { fields },
            simple_impl_attr_kit::ParsedValidationRule::AtLeastOneOf { fields } => {
                Self::AtLeastOneOf { fields }
            }
            simple_impl_attr_kit::ParsedValidationRule::CustomFunction { function_path } => {
                Self::CustomFunction { function_path }
            }
        }
    }
}

impl From<simple_impl_attr_kit::ParsedValidationSpec> for ValidationSpec {
    fn from(spec: simple_impl_attr_kit::ParsedValidationSpec) -> Self {
        Self::from_rules(spec.rules().iter().cloned().map(ValidationRule::from))
    }
}
