use crate::{error::Result, CompilerInput, CompilerOutput};

/// The result of a `solc` process bundled with its `Solc` and `CompilerInput`
type CompileElement<T> = (Result<CompilerOutput>, T, CompilerInput);

/// The bundled output of multiple `solc` processes.
#[derive(Debug)]
pub struct CompiledMany<T> {
    outputs: Vec<CompileElement<T>>,
}

impl<T> CompiledMany<T> {
    pub fn new(outputs: Vec<CompileElement<T>>) -> Self {
        Self { outputs }
    }

    /// Returns an iterator over all output elements
    pub fn outputs(&self) -> impl Iterator<Item = &CompileElement<T>> {
        self.outputs.iter()
    }

    /// Returns an iterator over all output elements
    pub fn into_outputs(self) -> impl Iterator<Item = CompileElement<T>> {
        self.outputs.into_iter()
    }

    /// Returns all `CompilerOutput` or the first error that occurred
    pub fn flattened(self) -> Result<Vec<CompilerOutput>> {
        self.into_iter().collect()
    }
}

impl<T> IntoIterator for CompiledMany<T> {
    type Item = Result<CompilerOutput>;
    type IntoIter = std::vec::IntoIter<Result<CompilerOutput>>;

    fn into_iter(self) -> Self::IntoIter {
        self.outputs.into_iter().map(|(res, _, _)| res).collect::<Vec<_>>().into_iter()
    }
}
