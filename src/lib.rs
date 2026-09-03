//! Shared core model/helper crate for `simple_impl_derive`.
//!
//! This crate is intentionally a normal Rust library, not a proc-macro crate.
//! It holds the semantic models and helper functions that proc-macro expansion
//! code can target and test without growing the proc-macro entry crate forever.

pub mod composite_contract;
pub mod composite_model;
pub mod model;
pub mod order;
pub mod ty;
pub mod validation;

pub use composite_contract::{
    CompositeEntryContract, CompositeShellContract, CompositeSurfaceContract,
};
pub use composite_model::{CompositeEntry, CompositeEntryKind, CompositeRootSpec};
pub use model::{
    BuilderCfg, BuilderKind, ExpandMode, FieldInfo, PosMode, ShellCfg, ShellFieldCfg,
};
pub use order::Order;
pub use ty::{
    type_is_bool, type_is_string, type_is_uint, type_option_inner, type_option_vec_inner,
    type_vec_inner,
};
pub use validation::{ValidationRule, ValidationSpec};
