//! Layout-neutral composite command contract models.
//!
//! All declaration styles should normalize into these contract structures before
//! planning or quote emission. Syntax can vary; contract formation should not.

use std::collections::BTreeSet;

use crate::{CompositeEntry, CompositeEntryKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeEntryContract {
    pub kind: CompositeEntryKind,
    pub method: String,
    pub segment: String,
    pub ty: String,
    pub init_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeShellContract {
    pub program: String,
    pub entries: Vec<CompositeEntryContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeSurfaceContract {
    pub segment: String,
    pub entries: Vec<CompositeEntryContract>,
}

impl CompositeEntryContract {
    pub fn new(
        kind: CompositeEntryKind,
        method: impl Into<String>,
        segment: impl Into<String>,
        ty: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            method: method.into(),
            segment: segment.into(),
            ty: ty.into(),
            init_args: Vec::new(),
        }
    }

    pub fn command(
        method: impl Into<String>,
        segment: impl Into<String>,
        ty: impl Into<String>,
    ) -> Self {
        Self::new(CompositeEntryKind::Command, method, segment, ty)
    }

    pub fn surface(
        method: impl Into<String>,
        segment: impl Into<String>,
        ty: impl Into<String>,
    ) -> Self {
        Self::new(CompositeEntryKind::Surface, method, segment, ty)
    }

    pub fn with_init_args<I, S>(mut self, init_args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.init_args = init_args.into_iter().map(Into::into).collect();
        self
    }

    pub fn from_entry(entry: &CompositeEntry) -> Self {
        Self {
            kind: entry.kind,
            method: entry.method.to_string(),
            segment: entry.segment.clone(),
            ty: entry.ty_tokens().to_string(),
            init_args: entry.init_args.iter().map(ToString::to_string).collect(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        require_non_empty(&self.method, "method")?;
        require_non_empty(&self.segment, "segment")?;
        require_non_empty(&self.ty, "ty")?;

        let mut seen = BTreeSet::new();
        for init_arg in &self.init_args {
            require_non_empty(init_arg, "init arg")?;
            if !seen.insert(init_arg) {
                return Err(format!("duplicate init arg `{init_arg}`"));
            }
        }

        Ok(())
    }

    pub fn is_command(&self) -> bool {
        self.kind == CompositeEntryKind::Command
    }

    pub fn is_surface(&self) -> bool {
        self.kind == CompositeEntryKind::Surface
    }
}

impl CompositeShellContract {
    pub fn new(program: impl Into<String>, entries: Vec<CompositeEntryContract>) -> Self {
        Self {
            program: program.into(),
            entries,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        require_non_empty(&self.program, "program")?;
        validate_entries(&self.entries)
    }
}

impl CompositeSurfaceContract {
    pub fn new(segment: impl Into<String>, entries: Vec<CompositeEntryContract>) -> Self {
        Self {
            segment: segment.into(),
            entries,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        require_non_empty(&self.segment, "surface segment")?;
        validate_entries(&self.entries)
    }
}

fn validate_entries(entries: &[CompositeEntryContract]) -> Result<(), String> {
    if entries.is_empty() {
        return Err("composite contract requires at least one entry".to_string());
    }

    let mut methods = BTreeSet::new();
    for entry in entries {
        entry.validate()?;
        if !methods.insert(&entry.method) {
            return Err(format!("duplicate composite method `{}`", entry.method));
        }
    }

    Ok(())
}

fn require_non_empty(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("composite contract requires non-empty {label}"))
    } else {
        Ok(())
    }
}
