use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
pub struct Config {
    pub package: Package,
    pub dependencies: BTreeMap<String, Dependency>,
}

impl Config {
    pub fn new() -> Self {
        Self { package: Package::new(), dependencies: BTreeMap::new() }
    }

    // Local paths are usually relative and are discouraged when sharing libraries
    // It is better to separate these into different packages.
    pub fn has_local_path(&self) -> bool {
        let mut has_local_path = false;
        for dep in self.dependencies.values() {
            if let Dependency::Path { .. } = dep {
                has_local_path = true;
                break;
            }
        }
        has_local_path
    }
}

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
pub struct Package {
    // Note: a package name is not needed unless there is a registry
    pub authors: Vec<String>,
    // If not compiler version is supplied, the latest is used
    // For now, we state that all packages must be compiled under the same
    // compiler version.
    // We also state that ACIR and the compiler will upgrade in lockstep.
    // so you will not need to supply an ACIR and compiler version
    pub compiler_version: Option<String>,
    pub backend: Option<String>,
    pub license: Option<String>,
}

impl Package {
    pub fn new() -> Self {
        Self {
            authors: Vec::new(),
            compiler_version: Some("0.1".to_string()),
            backend: None,
            license: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(untagged)]
/// Enum representing the different types of ways to
/// supply a source for the dependency
pub enum Dependency {
    Github { git: String, tag: String },
    Path { path: String },
}

#[test]
fn parse_standard_toml() {
    let src = r#"

        [package]
        authors = ["kev", "foo"]
        compiler_version = "0.1"

        [dependencies]
        rand = { tag = "next", git = "https://github.com/rust-lang-nursery/rand" }
        cool = { tag = "next", git = "https://github.com/rust-lang-nursery/rand" }
        hello = { path = "./noir_driver" }
    "#;

    let parsed_config: Result<Config, _> = toml::from_str(src);
    assert!(parsed_config.is_ok());
}
