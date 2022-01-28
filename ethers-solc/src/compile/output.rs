//! The output of a compiled project

use crate::{
    artifacts::{Error, SourceFile, VersionedContract, VersionedContracts},
    ArtifactOutput, CompilerOutput, WrittenArtifacts,
};
use semver::Version;
use std::{collections::BTreeMap, path::PathBuf};

/// Contains a mixture of already compiled/cached artifacts and the input set of sources that still
/// need to be compiled.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProjectCompileOutput2<T: ArtifactOutput> {
    /// contains the aggregated `CompilerOutput`
    ///
    /// See [`CompilerSources::compile`]
    pub(crate) compiler_output: AggregatedCompilerOutput,
    /// all artifact files from `output` that were written
    pub(crate) written_artifacts: WrittenArtifacts<T::Artifact>,
    /// All artifacts that were read from cache
    pub(crate) cached_artifacts: BTreeMap<PathBuf, T::Artifact>,
    /// errors that should be omitted
    pub(crate) ignored_error_codes: Vec<u64>,
}

impl<T: ArtifactOutput> ProjectCompileOutput2<T> {
    /// Get the (merged) solc compiler output
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use ethers_solc::artifacts::Contract;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: BTreeMap<String, Contract> =
    ///     project.compile().unwrap().output().contracts_into_iter().collect();
    /// ```
    pub fn output(self) -> AggregatedCompilerOutput {
        self.compiler_output
    }

    /// Whether this type has a compiler output
    pub fn has_compiled_contracts(&self) -> bool {
        self.compiler_output.is_empty()
    }

    /// Whether there were errors
    pub fn has_compiler_errors(&self) -> bool {
        self.compiler_output.has_error()
    }

    /// Whether there were warnings
    pub fn has_compiler_warnings(&self) -> bool {
        self.compiler_output
            .as_ref()
            .map(|o| o.has_warning(&self.ignored_error_codes))
            .unwrap_or_default()
    }
}
/// The aggregated output of (multiple) compile jobs
///
/// This is effectively a solc version aware `CompilerOutput`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AggregatedCompilerOutput {
    /// all errors from all `CompilerOutput`
    pub errors: Vec<Error>,
    /// All source files
    pub sources: BTreeMap<String, SourceFile>,
    /// All compiled contracts combined with the solc version used to compile them
    pub contracts: VersionedContracts,
}

impl AggregatedCompilerOutput {
    /// Whether the output contains a compiler error
    pub fn has_error(&self) -> bool {
        self.errors.iter().any(|err| err.severity.is_error())
    }

    /// Whether the output contains a compiler warning
    pub fn has_warning(&self, ignored_error_codes: &[u64]) -> bool {
        self.errors.iter().any(|err| {
            if err.severity.is_warning() {
                err.error_code.as_ref().map_or(false, |code| !ignored_error_codes.contains(code))
            } else {
                false
            }
        })
    }

    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }

    pub fn extend_all<I>(&mut self, out: I)
    where
        I: IntoIterator<Item = (Version, CompilerOutput)>,
    {
        for (v, o) in out {
            self.extend(v, o)
        }
    }

    /// adds a new `CompilerOutput` to the aggregated output
    pub fn extend(&mut self, version: Version, output: CompilerOutput) {
        self.errors.extend(output.errors);
        self.sources.extend(output.sources);

        for (file_name, new_contracts) in output.contracts {
            let contracts = self.contracts.entry(file_name).or_default();
            for (contract_name, contract) in new_contracts {
                let versioned = contracts.entry(contract_name).or_default();
                versioned.push(VersionedContract { contract, version: version.clone() });
            }
        }
    }
}
