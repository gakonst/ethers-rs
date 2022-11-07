use cargo_metadata::MetadataCommand;
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
/// This determines
///   - `ethers_*` idents if `ethers-core`, `ethers-contract`, `ethers-providers`  are present in
///     the manifest or the `ethers` is _not_ present
///   - `ethers::*` otherwise
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
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR");

    // if there is no cargo manifest, default to `ethers::`-style imports.
    let manifest_dir = if let Ok(manifest_dir) = manifest_dir {
        manifest_dir
    } else {
        return ("ethers::core", "ethers::contract", "ethers::providers")
    };

    // check if the lock file exists, if it's missing we need to clean up afterward
    let lock_file = format!("{manifest_dir}/Cargo.lock");
    let needs_lock_file_cleanup = !std::path::Path::new(&lock_file).exists();

    let res = MetadataCommand::new()
        .manifest_path(&format!("{manifest_dir}/Cargo.toml"))
        .exec()
        .ok()
        .and_then(|metadata| {
            metadata.root_package().and_then(|pkg| {
                let sub_crates = Some(("ethers_core", "ethers_contract", "ethers_providers"));

                // Note(mattsse): this is super hacky but required in order to compile and test
                // ethers' internal crates
                if [
                    "ethers-contract",
                    "ethers-derive-eip712",
                    "ethers-signers",
                    "ethers-middleware",
                    "ethers-solc",
                ]
                .contains(&pkg.name.as_str())
                {
                    return sub_crates
                }

                let mut has_ethers_core = false;
                let mut has_ethers_contract = false;
                let mut has_ethers_providers = false;

                for dep in pkg.dependencies.iter() {
                    match dep.name.as_str() {
                        "ethers-core" => {
                            has_ethers_core = true;
                        }
                        "ethers-contract" => {
                            has_ethers_contract = true;
                        }
                        "ethers-providers" => {
                            has_ethers_providers = true;
                        }
                        "ethers" => return None,
                        _ => {}
                    }
                }

                if has_ethers_core && has_ethers_contract && has_ethers_providers {
                    return sub_crates
                }

                None
            })
        })
        .unwrap_or(("ethers::core", "ethers::contract", "ethers::providers"));

    if needs_lock_file_cleanup {
        // delete the `Cargo.lock` file that was created by `cargo metadata`
        // if the package is not part of a workspace
        let _ = std::fs::remove_file(lock_file);
    }

    res
}
