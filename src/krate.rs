use std::collections::HashMap;

use semver::{Version, VersionReq};
use serde::{Serialize, Deserialize};

/// Represents a crate.  
/// This is what's stored in the crate index.  
/// Note that this structs represents only a specific version of a crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Crate {
    /// The name of the crate.
    pub name: String,
    /// The version of the crate.
    pub vers: Version,
    /// The dependencies of the crate.
    pub deps: Vec<Dependency>,
    /// The SHA256 hash of the crate.
    pub cksum: String,
    /// The available features of the crates and what they enable.
    pub features: HashMap<String, Vec<String>>,
    /// Is the crate yanked.
    pub yanked: Option<bool>,
}

/// Represents a crate dependency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// The name of the dependency.
    pub name: String,
    /// The version requirement for the dependency (eg. "^1.2.0").
    pub req: VersionReq,
    /// The features requested for the dependency.
    pub features: Vec<String>,
    /// Is the dependency optional.
    pub optional: bool,
    /// Are the default features requested for the dependency.
    pub default_features: bool,
    /// The target of the dependency.
    pub target: Option<String>,
    /// The kind of the dependency ("normal", "build" or "dev").
    pub kind: Option<DependencyKind>,
}

/// Represents the different kinds of dependencies.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    /// A normal dependency.
    Normal,
    /// A build dependency.
    Build,
    /// A developement dependency.
    Dev,
}
