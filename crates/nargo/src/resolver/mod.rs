use std::path::{Path, PathBuf};

use acvm::Language;
use noirc_driver::Driver;

use crate::{git::clone_git_repo, manifest::Dependency};

use self::generic_resolver::{CachedDep, Resolver};

pub(crate) use self::generic_resolver::DependencyResolutionError;

pub(crate) mod generic_resolver;

/// Resolves a manifest by either downloading the necessary git repo or it uses the repo on the cache.
/// Downloading will be recursive, so if a package contains packages we need to download those too
pub(crate) struct CliResolver;

impl CliResolver {
    /// Returns a `Driver` which can be used to compile the crate.
    pub(crate) fn resolve_root_manifest(
        dir_path: &Path,
        np_language: Language,
    ) -> Result<Driver, DependencyResolutionError> {
        let manifest_path = super::find_package_manifest(dir_path)?;
        let manifest = super::manifest::parse(manifest_path)?;
        let (crate_entrypoint, crate_type) = super::lib_or_bin(dir_path)?;

        Resolver::resolve_root_manifest(
            manifest,
            &crate_entrypoint,
            crate_type,
            np_language,
            cache_dep,
        )
    }
}

/// If the dependency is remote, download the dependency
/// and return the directory path along with the metadata
/// Needed to fill the CachedDep struct
///
/// If it's a local path, the same applies, however it will not
/// be downloaded
fn cache_dep(dep: &Dependency) -> Result<(PathBuf, CachedDep), DependencyResolutionError> {
    fn retrieve_meta(
        dir_path: &Path,
        remote: bool,
    ) -> Result<CachedDep, DependencyResolutionError> {
        let (entry_path, crate_type) = super::lib_or_bin(dir_path)?;
        let manifest_path = super::find_package_manifest(dir_path)?;
        let manifest = super::manifest::parse(manifest_path)?;
        Ok(CachedDep { entry_path, crate_type, manifest, remote })
    }

    match dep {
        Dependency::Github { git, tag } => {
            let dir_path = clone_git_repo(git, tag).map_err(DependencyResolutionError::GitError)?;
            let meta = retrieve_meta(&dir_path, true)?;
            Ok((dir_path, meta))
        }
        Dependency::Path { path } => {
            let dir_path = std::path::PathBuf::from(path);
            let meta = retrieve_meta(&dir_path, false)?;
            Ok((dir_path, meta))
        }
    }
}
