use cargo_metadata::{DependencyKind, MetadataCommand};
use once_cell::sync::Lazy;
use syn::Path;

/// See `determine_ethers_crates`
///
/// This ensures that the `MetadataCommand` is only run once
static ETHERS_CRATES: Lazy<(&'static str, &'static str, &'static str)> =
    Lazy::new(determine_ethers_crates);

/// Convenience function to turn the `ethers_core` name in `ETHERS_CRATE` into a `Path`
pub fn ethers_core_crate() -> Path {
    syn::parse_str(ETHERS_CRATES.0).expect("valid path; qed")
}
/// Convenience function to turn the `ethers_contract` name in `ETHERS_CRATE` into an `Path`
pub fn ethers_contract_crate() -> Path {
    syn::parse_str(ETHERS_CRATES.1).expect("valid path; qed")
}
pub fn ethers_providers_crate() -> Path {
    syn::parse_str(ETHERS_CRATES.2).expect("valid path; qed")
}

/// The crates name to use when deriving macros: (`core`, `contract`)
///
/// We try to determine which crate ident to use based on the dependencies of
/// the project in which the macro is used. This is useful because the macros,
/// like `EthEvent` are provided by the `ethers-contract` crate which depends on
/// `ethers_core`. Most commonly `ethers` will be used as dependency which
/// reexports all the different crates, essentially `ethers::core` is
/// `ethers_core` So depending on the dependency used `ethers` ors `ethers_core
/// | ethers_contract`, we need to use the fitting crate ident when expand the
/// macros This will attempt to parse the current `Cargo.toml` and check the
/// ethers related dependencies.
///
/// This process is a bit hacky, we run `cargo metadata` internally which
/// resolves the current package but creates a new `Cargo.lock` file in the
/// process. This is not a problem for regular workspaces but becomes an issue
/// during publishing with `cargo publish` if the project does not ignore
/// `Cargo.lock` in `.gitignore`, because then cargo can't proceed with
/// publishing the crate because the created `Cargo.lock` leads to a modified
/// workspace, not the `CARGO_MANIFEST_DIR` but the workspace `cargo publish`
/// created in `./target/package/..`. Therefore we check prior to executing
/// `cargo metadata` if a `Cargo.lock` file exists and delete it afterwards if
/// it was created by `cargo metadata`.
pub fn determine_ethers_crates() -> (&'static str, &'static str, &'static str) {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("No Manifest found");

    // check if the lock file exists, if it's missing we need to clean up afterward
    let lock_file = format!("{}/Cargo.lock", manifest_dir);
    let needs_lock_file_cleanup = !std::path::Path::new(&lock_file).exists();

    let res = MetadataCommand::new()
        .manifest_path(&format!("{}/Cargo.toml", manifest_dir))
        .exec()
        .ok()
        .and_then(|metadata| {
            metadata.root_package().and_then(|pkg| {
                pkg.dependencies.iter().filter(|dep| dep.kind == DependencyKind::Normal).find_map(
                    |dep| {
                        (dep.name == "ethers")
                            .then(|| ("ethers::core", "ethers::contract", "ethers::providers"))
                    },
                )
            })
        })
        .unwrap_or(("ethers_core", "ethers_contract", "ethers_providers"));

    if needs_lock_file_cleanup {
        // delete the `Cargo.lock` file that was created by `cargo metadata`
        // if the package is not part of a workspace
        let _ = std::fs::remove_file(lock_file);
    }

    res
}
