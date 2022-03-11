//! Manages compiling a solidity `Project`
//!
//! 

use traits::{Compiler, Result};
use rayon::prelude::*;
use std::collections::btree_map::BTreeMap;

#[derive(Debug)]
pub struct Pipeline {}

impl Pipeline {
    /// Does basic preprocessing
    ///   - sets proper source unit names
    ///   - check cache
    fn preprocess(self, T: Compiler) -> Result<PreprocessedState<'a, T>> {
        let Self { edges, project, mut sources } = self;

        let mut cache = ArtifactsCache::new(project, edges)?;
        // retain and compile only dirty sources and all their imports
        sources = sources.filtered(&mut cache);

        Ok(PreprocessedState { sources, cache })
    }
}