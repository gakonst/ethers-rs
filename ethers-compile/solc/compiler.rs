//! Manages compiling a solidity `Project`
//!
//! 

use crate::{Compiler, Result};
use rayon::prelude::*;
use std::collections::btree_map::BTreeMap;

#[derive(Debug)]
pub struct SolcCompiler<'a, T> {
    /// Contains the relationship of the source files and their imports
    edges: GraphEdges,
    project: &'a Project<T>,
    /// how to compile all the sources
    sources: CompilerSources,
}

#[derive(Debug)]
pub enum SolcCompilerError {
    Unknown
};

impl Compiler<CompilerInput, ProjectCompileOutput, SolcCompilerError> for SolcCompiler {
    /// Compiles all the sources of the `Project` in the appropriate mode
    ///
    /// If caching is enabled, the sources are filtered and only _dirty_ sources are recompiled.
    ///
    /// The output of the compile process can be a mix of reused artifacts and freshly compiled
    /// `Contract`s
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let output = project.compile().unwrap();
    /// ```
    fn compile(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
        Pipeline::preprocess(&self)?.compile(&self)?.write_artifacts(&self)?.write_cache(&self)
    }
};