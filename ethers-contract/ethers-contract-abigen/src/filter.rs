//! Filtering support for contracts used in [`Abigen`][crate::Abigen]

use regex::Regex;
use std::collections::HashSet;

/// Used to filter contracts that should be _included_ in the abigen generation.
#[derive(Debug, Default, Clone)]
pub enum ContractFilter {
    /// Include all contracts
    #[default]
    All,
    /// Only include contracts that match the filter
    Select(SelectContracts),
    /// Only include contracts that _don't_ match the filter
    Exclude(ExcludeContracts),
}

// === impl ContractFilter ===

impl ContractFilter {
    /// Returns whether to include the contract with the given `name`
    pub fn is_match(&self, name: impl AsRef<str>) -> bool {
        match self {
            ContractFilter::All => true,
            ContractFilter::Select(f) => f.is_match(name),
            ContractFilter::Exclude(f) => !f.is_match(name),
        }
    }
}

impl From<SelectContracts> for ContractFilter {
    fn from(f: SelectContracts) -> Self {
        ContractFilter::Select(f)
    }
}

impl From<ExcludeContracts> for ContractFilter {
    fn from(f: ExcludeContracts) -> Self {
        ContractFilter::Exclude(f)
    }
}

macro_rules! impl_filter {
    ($name:ident) => {
        impl $name {
            /// Adds an exact name to the filter
            pub fn add_name<T: Into<String>>(mut self, arg: T) -> Self {
                self.exact.insert(arg.into());
                self
            }

            /// Adds multiple exact names to the filter
            pub fn extend_names<I, S>(mut self, name: I) -> Self
            where
                I: IntoIterator<Item = S>,
                S: Into<String>,
            {
                for arg in name {
                    self = self.add_name(arg);
                }
                self
            }

            /// Adds the regex to use
            ///
            /// # Panics
            ///
            /// If `pattern` is an invalid `Regex`
            pub fn add_regex(mut self, re: Regex) -> Self {
                self.patterns.push(re);
                self
            }

            /// Adds multiple exact names to the filter
            pub fn extend_regex<I, S>(mut self, regexes: I) -> Self
            where
                I: IntoIterator<Item = S>,
                S: Into<Regex>,
            {
                for re in regexes {
                    self = self.add_regex(re.into());
                }
                self
            }

            /// Sets the pattern to use
            ///
            /// # Panics
            ///
            /// If `pattern` is an invalid `Regex`
            pub fn add_pattern(self, pattern: impl AsRef<str>) -> Self {
                self.try_add_pattern(pattern).unwrap()
            }

            /// Sets the pattern to use
            pub fn try_add_pattern(mut self, s: impl AsRef<str>) -> Result<Self, regex::Error> {
                self.patterns.push(Regex::new(s.as_ref())?);
                Ok(self)
            }

            /// Adds multiple patterns to the filter
            ///
            /// # Panics
            ///
            /// If `pattern` is an invalid `Regex`
            pub fn extend_pattern<I, S>(self, patterns: I) -> Self
            where
                I: IntoIterator<Item = S>,
                S: AsRef<str>,
            {
                self.try_extend_pattern(patterns).unwrap()
            }

            /// Adds multiple patterns to the filter
            ///
            /// # Panics
            ///
            /// If `pattern` is an invalid `Regex`
            pub fn try_extend_pattern<I, S>(mut self, patterns: I) -> Result<Self, regex::Error>
            where
                I: IntoIterator<Item = S>,
                S: AsRef<str>,
            {
                for p in patterns {
                    self = self.try_add_pattern(p)?;
                }
                Ok(self)
            }

            /// Returns true whether the `name` matches the filter
            pub fn is_match(&self, name: impl AsRef<str>) -> bool {
                let name = name.as_ref();
                if self.exact.contains(name) {
                    return true
                }
                self.patterns.iter().any(|re| re.is_match(name))
            }
        }
    };
}

/// A Contract Filter that only includes certain contracts.
///
/// **Note:**: matching by exact name and via regex stacks
///
/// This is the inverse of `ExcludeContracts`
#[derive(Debug, Clone, Default)]
pub struct SelectContracts {
    /// Include contracts based on their exact name
    exact: HashSet<String>,
    /// Include contracts if their name matches a pattern
    patterns: Vec<Regex>,
}

/// A Contract Filter that exclude certain contracts
///
/// **Note:**: matching by exact name and via regex stacks
///
/// This is the inverse of `SelectContracts`
#[derive(Debug, Clone, Default)]
pub struct ExcludeContracts {
    /// Exclude contracts based on their exact name
    exact: HashSet<String>,
    /// Exclude contracts if their name matches any pattern
    patterns: Vec<Regex>,
}

impl_filter!(SelectContracts);
impl_filter!(ExcludeContracts);
