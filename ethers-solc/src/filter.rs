//! Types to apply filter to input types

use crate::{
    artifacts::{output_selection::OutputSelection, Settings},
    Source, Sources,
};
use std::{
    collections::BTreeMap,
    fmt,
    fmt::Formatter,
    path::{Path, PathBuf},
};

/// A predicate property that determines whether a file satisfies a certain condition
pub trait FileFilter {
    /// The predicate function that should return if the given `file` should be included.
    fn is_match(&self, file: &Path) -> bool;
}

impl<F> FileFilter for F
where
    F: Fn(&Path) -> bool,
{
    fn is_match(&self, file: &Path) -> bool {
        (self)(file)
    }
}

/// An [FileFilter] that matches all solidity files that end with `.t.sol`
#[derive(Default)]
pub struct TestFileFilter {
    _priv: (),
}

impl fmt::Debug for TestFileFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestFileFilter").finish()
    }
}

impl fmt::Display for TestFileFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("TestFileFilter")
    }
}

impl FileFilter for TestFileFilter {
    fn is_match(&self, file: &Path) -> bool {
        file.file_name().and_then(|s| s.to_str()).map(|s| s.ends_with(".t.sol")).unwrap_or_default()
    }
}

/// A type that can apply a filter to a set of preprocessed [FilteredSources] in order to set sparse
/// output for specific files
pub enum SparseOutputFileFilter {
    /// Sets the configured [OutputSelection] for dirty files only.
    ///
    /// In other words, we request the output of solc only for files that have been detected as
    /// _dirty_.
    AllDirty,
    /// Apply an additional filter to [FilteredSources] to
    Custom(Box<dyn FileFilter>),
}

impl SparseOutputFileFilter {
    /// While solc needs all the files to compile the actual _dirty_ files, we can tell solc to
    /// output everything for those dirty files as currently configured in the settings, but output
    /// nothing for the other files that are _not_ dirty.
    ///
    /// This will modify the [OutputSelection] of the [Settings] so that we explicitly select the
    /// files' output based on their state.
    pub fn sparse_sources(&self, sources: FilteredSources, settings: &mut Settings) -> Sources {
        fn apply(
            sources: &FilteredSources,
            settings: &mut Settings,
            f: impl Fn(&PathBuf, &FilteredSource) -> bool,
        ) {
            let selection = settings
                .output_selection
                .as_mut()
                .remove("*")
                .unwrap_or_else(OutputSelection::default_file_output_selection);

            for (file, source) in sources.0.iter() {
                if f(file, source) {
                    settings
                        .output_selection
                        .as_mut()
                        .insert(format!("{}", file.display()), selection.clone());
                } else {
                    tracing::trace!("using pruned output selection for {}", file.display());
                    settings.output_selection.as_mut().insert(
                        format!("{}", file.display()),
                        OutputSelection::empty_file_output_select(),
                    );
                }
            }
        }

        match self {
            SparseOutputFileFilter::AllDirty => {
                if !sources.all_dirty() {
                    // settings can be optimized
                    tracing::trace!(
                        "optimizing output selection for {}/{} sources",
                        sources.clean().count(),
                        sources.len()
                    );
                    apply(&sources, settings, |_, source| source.is_dirty())
                }
            }
            SparseOutputFileFilter::Custom(f) => {
                tracing::trace!("optimizing output selection with custom filter",);
                apply(&sources, settings, |p, source| source.is_dirty() && f.is_match(p));
            }
        };
        sources.into()
    }
}

impl From<Box<dyn FileFilter>> for SparseOutputFileFilter {
    fn from(f: Box<dyn FileFilter>) -> Self {
        SparseOutputFileFilter::Custom(f)
    }
}

impl Default for SparseOutputFileFilter {
    fn default() -> Self {
        SparseOutputFileFilter::AllDirty
    }
}

impl fmt::Debug for SparseOutputFileFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SparseOutputFileFilter::AllDirty => f.write_str("AllDirty"),
            SparseOutputFileFilter::Custom(_) => f.write_str("Custom"),
        }
    }
}

/// Container type for a set of [FilteredSource]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FilteredSources(pub BTreeMap<PathBuf, FilteredSource>);

impl FilteredSources {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if all files are dirty
    pub fn all_dirty(&self) -> bool {
        self.0.values().all(|s| s.is_dirty())
    }

    /// Returns all entries that are dirty
    pub fn dirty(&self) -> impl Iterator<Item = (&PathBuf, &FilteredSource)> + '_ {
        self.0.iter().filter(|(_, s)| s.is_dirty())
    }

    /// Returns all entries that are clean
    pub fn clean(&self) -> impl Iterator<Item = (&PathBuf, &FilteredSource)> + '_ {
        self.0.iter().filter(|(_, s)| !s.is_dirty())
    }

    /// Returns all dirty files
    pub fn dirty_files(&self) -> impl Iterator<Item = &PathBuf> + fmt::Debug + '_ {
        self.0.iter().filter_map(|(k, s)| s.is_dirty().then(|| k))
    }
}

impl From<FilteredSources> for Sources {
    fn from(sources: FilteredSources) -> Self {
        sources.0.into_iter().map(|(k, v)| (k, v.into_source())).collect()
    }
}

impl From<Sources> for FilteredSources {
    fn from(s: Sources) -> Self {
        FilteredSources(s.into_iter().map(|(key, val)| (key, FilteredSource::Dirty(val))).collect())
    }
}

impl From<BTreeMap<PathBuf, FilteredSource>> for FilteredSources {
    fn from(s: BTreeMap<PathBuf, FilteredSource>) -> Self {
        FilteredSources(s)
    }
}

impl AsRef<BTreeMap<PathBuf, FilteredSource>> for FilteredSources {
    fn as_ref(&self) -> &BTreeMap<PathBuf, FilteredSource> {
        &self.0
    }
}

impl AsMut<BTreeMap<PathBuf, FilteredSource>> for FilteredSources {
    fn as_mut(&mut self) -> &mut BTreeMap<PathBuf, FilteredSource> {
        &mut self.0
    }
}

/// Represents the state of a filtered [Source]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FilteredSource {
    /// A source that fits the _dirty_ criteria
    Dirty(Source),
    /// A source that does _not_ fit the _dirty_ criteria but is included in the filtered set
    /// because a _dirty_ file pulls it in, either directly on indirectly.
    Clean(Source),
}

impl FilteredSource {
    /// Returns the underlying source
    pub fn source(&self) -> &Source {
        match self {
            FilteredSource::Dirty(s) => s,
            FilteredSource::Clean(s) => s,
        }
    }

    /// Consumes the type and returns the underlying source
    pub fn into_source(self) -> Source {
        match self {
            FilteredSource::Dirty(s) => s,
            FilteredSource::Clean(s) => s,
        }
    }

    /// Whether this file is actually dirt
    pub fn is_dirty(&self) -> bool {
        matches!(self, FilteredSource::Dirty(_))
    }
}

/// Helper type that determines the state of a source file
#[derive(Debug)]
pub struct FilteredSourceInfo {
    /// path to the source file
    pub file: PathBuf,
    /// contents of the file
    pub source: Source,
    /// idx in the [GraphEdges]
    pub idx: usize,
    /// whether this file is actually dirty
    ///
    /// See also [ArtifactsCacheInner::is_dirty()]
    pub dirty: bool,
}
