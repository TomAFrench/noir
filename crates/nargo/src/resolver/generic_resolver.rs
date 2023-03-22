use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use acvm::Language;
use noirc_driver::Driver;
use noirc_frontend::graph::{CrateId, CrateName, CrateType};
use thiserror::Error;

use crate::{
    manifest::{Dependency, PackageManifest},
    InvalidPackageError,
};

/// Errors covering situations where a crate's dependency tree cannot be resolved.
#[derive(Debug, Error)]
pub(crate) enum DependencyResolutionError {
    /// Encountered error while downloading git repository.
    #[error("{0}")]
    GitError(String),

    /// Attempted to depend on a binary crate.
    #[error("dependency {dep_pkg_name} is a binary package and so it cannot be depended upon.")]
    BinaryDependency { dep_pkg_name: String },

    /// Attempted to depend on remote crate which has a local dependency.
    /// We have no guarantees that this local dependency will be available so must error.
    #[error("remote(git) dependency has a local dependency.\ndependency located at {}", dependency_path.display())]
    RemoteDepWithLocalDep { dependency_path: PathBuf },

    /// Dependency is not a valid crate
    #[error(transparent)]
    MalformedDependency(#[from] InvalidPackageError),
}

#[derive(Debug, Clone)]
pub(super) struct CachedDep {
    pub(super) entry_path: PathBuf,
    pub(super) crate_type: CrateType,
    pub(super) manifest: PackageManifest,
    // Whether the dependency came from
    // a remote dependency
    pub(super) remote: bool,
}

// TODO: We'll probably need to change this to a `Fn` type at some point so we can pass closures.
type DependencyFetcher =
    fn(dep: &Dependency) -> Result<(PathBuf, CachedDep), DependencyResolutionError>;

/// A generic implementation of the Nargo dependency resolver. `Resolver` implements the core logic for how
/// to explore the dependency tree and build the `Driver` with which to compile the Noir program, however
///
pub(super) struct Resolver<'a> {
    driver: &'a mut Driver,
}

impl<'a> Resolver<'a> {
    fn with_driver(driver: &mut Driver) -> Resolver {
        Resolver { driver }
    }

    /// Returns a `Driver` which can be used to compile the crate.
    pub(super) fn resolve_root_manifest(
        manifest: PackageManifest,
        crate_entrypoint: &Path,
        crate_type: CrateType,
        np_language: Language,
        fetch_dependency: DependencyFetcher,
    ) -> Result<Driver, DependencyResolutionError> {
        let mut driver = Driver::new(&np_language);
        let crate_id = driver.create_local_crate(crate_entrypoint, crate_type);

        let mut resolver = Self::with_driver(&mut driver);
        resolver.resolve_manifest(crate_id, manifest, fetch_dependency)?;

        add_std_lib(&mut driver);
        Ok(driver)
    }

    // TODO: Need to solve the case of a project trying to use itself as a dep
    /// Resolves a package manifest by recursively resolving the dependencies in the manifest.
    fn resolve_manifest(
        &mut self,
        parent_crate: CrateId,
        manifest: PackageManifest,
        fetch_dependency: DependencyFetcher,
    ) -> Result<(), DependencyResolutionError> {
        let mut cached_packages: HashMap<PathBuf, (CrateId, CachedDep)> = HashMap::new();

        // First download and add these top level dependencies crates to the Driver
        for (dep_pkg_name, pkg_src) in manifest.dependencies.iter() {
            let (dir_path, dep_meta) = fetch_dependency(pkg_src)?;

            let (entry_path, crate_type) = (&dep_meta.entry_path, &dep_meta.crate_type);

            if crate_type == &CrateType::Binary {
                return Err(DependencyResolutionError::BinaryDependency {
                    dep_pkg_name: dep_pkg_name.to_string(),
                });
            }

            // Note: this call still relies on `fm` to insert the crate root into the file manager
            let crate_id = self.driver.create_non_local_crate(entry_path, *crate_type);

            self.driver.add_dep(parent_crate, crate_id, dep_pkg_name);

            cached_packages.insert(dir_path, (crate_id, dep_meta));
        }

        // Resolve all transitive dependencies
        for (dependency_path, (crate_id, dep_meta)) in cached_packages {
            if dep_meta.remote && manifest.has_local_path() {
                return Err(DependencyResolutionError::RemoteDepWithLocalDep { dependency_path });
            }
            self.resolve_manifest(crate_id, dep_meta.manifest, fetch_dependency)?;
        }
        Ok(())
    }
}

// This needs to be public to support the tests in `cli/mod.rs`.
pub(crate) fn add_std_lib(driver: &mut Driver) {
    let std_crate_name = "std";
    let path_to_std_lib_file = PathBuf::from(std_crate_name).join("lib.nr");
    let std_crate = driver.create_non_local_crate(path_to_std_lib_file, CrateType::Library);
    driver.propagate_dep(std_crate, &CrateName::new(std_crate_name).unwrap());
}
