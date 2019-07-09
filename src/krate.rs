use std::collections::HashMap;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

/// Represents a crate.
///
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
    /// Is the crate yanked.
    pub links: Option<String>,
}

/// Represents a crate dependency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// The name of the dependency.
    ///
    /// If the dependency is renamed, this is the new name.  
    /// The original name is specified in the `package` field.
    pub name: String,
    /// The version requirement for the dependency (eg. "^1.2.0").
    pub req: VersionReq,
    /// The features requested for the dependency.
    pub features: Vec<String>,
    /// Is the dependency optional.
    pub optional: bool,
    /// Whether the crates uses the default features of this dependency.
    pub default_features: bool,
    /// The target platform of the dependency.
    ///
    /// A string such as "cfg(windows)"
    pub target: Option<String>,
    /// The kind of the dependency ("normal", "build" or "dev").
    pub kind: Option<DependencyKind>,
    /// The URL of the index of the registry where this dependency is from.
    ///
    /// If not specified, it is assumed to come from the current registry.
    pub registry: Option<String>,
    /// If the dependency is renamed, this is the actual original crate name.
    ///
    /// If not specified, the dependency has not been renamed.
    pub package: Option<String>,
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
