//! Helpers to generate mock projects

use crate::{error::Result, remappings::Remapping, ProjectPathsConfig};
use rand::{self, seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

/// Represents a virtual project
#[derive(Serialize)]
pub struct MockProjectGenerator {
    /// how to name things
    #[serde(skip)]
    name_strategy: Box<dyn NamingStrategy + 'static>,
    /// id counter for a file
    next_file_id: usize,
    /// id counter for a file
    next_lib_id: usize,
    /// all files for the project
    files: Vec<MockFile>,
    /// all libraries
    libraries: Vec<MockLib>,
}

impl Default for MockProjectGenerator {
    fn default() -> Self {
        Self {
            name_strategy: Box::new(SimpleNamingStrategy::default()),
            next_file_id: 0,
            next_lib_id: 0,
            files: Default::default(),
            libraries: Default::default(),
        }
    }
}

impl MockProjectGenerator {
    /// Generate all solidity files and write under the paths config
    pub fn write_to(&self, paths: &ProjectPathsConfig, version: impl AsRef<str>) -> Result<()> {
        let version = version.as_ref();
        for file in self.files.iter() {
            let mut imports = Vec::with_capacity(file.imports.len());

            for import in file.imports.iter() {
                match *import {
                    MockImport::Internal(f) => {
                        imports.push(format!("import \"./{}.sol\";", self.files[f].name));
                    }
                    MockImport::External(lib, f) => {
                        imports.push(format!(
                            "import \"{}/{}.sol\";",
                            self.libraries[lib].name, self.files[f].name
                        ));
                    }
                }
            }

            let content = format!(
                r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity {};
{}
contract {} {{}}
            "#,
                version,
                imports.join("\n"),
                file.name
            );

            let mut target = if let Some(lib) = file.lib_id {
                paths.root.join("lib").join(&self.libraries[lib].name).join("src").join(&file.name)
            } else {
                paths.sources.join(&file.name)
            };
            target.set_extension("sol");

            super::create_contract_file(target, content)?;
        }

        Ok(())
    }

    /// Returns all the remappings for the project for the given root path
    pub fn remappings_at(&self, root: &Path) -> Vec<Remapping> {
        self.libraries
            .iter()
            .map(|lib| {
                let path = root.join("lib").join(&lib.name).join("src");
                format!("{}/={}/", lib.name, path.display()).parse().unwrap()
            })
            .collect()
    }

    /// Returns all the remappings for the project
    pub fn remappings(&self) -> Vec<Remapping> {
        self.libraries
            .iter()
            .map(|lib| format!("{0}/=lib/{0}/src/", lib.name).parse().unwrap())
            .collect()
    }

    /// Create a new project and populate it using the given settings
    pub fn new(settings: &MockProjectSettings) -> Self {
        let mut mock = Self::default();
        mock.populate(settings);
        mock
    }

    /// Generates a random project with random settings
    pub fn random() -> Self {
        let settings = MockProjectSettings::random();
        let mut mock = Self::default();
        mock.populate(&settings);
        mock
    }

    /// Adds sources and libraries and populates imports based on the settings
    pub fn populate(&mut self, settings: &MockProjectSettings) -> &mut Self {
        self.add_sources(settings.num_lib_files);
        for _ in 0..settings.num_libs {
            self.add_lib(settings.num_lib_files);
        }
        self.populate_imports(settings)
    }

    fn next_file_id(&mut self) -> usize {
        let next = self.next_file_id;
        self.next_file_id += 1;
        next
    }

    fn next_lib_id(&mut self) -> usize {
        let next = self.next_lib_id;
        self.next_lib_id += 1;
        next
    }

    /// Adds a new source file
    pub fn add_source(&mut self) -> &mut Self {
        let id = self.next_file_id();
        let name = self.name_strategy.new_source_file_name(id);
        let file = MockFile { id, name, imports: Default::default(), lib_id: None };
        self.files.push(file);
        self
    }

    /// Adds `num` new source files
    pub fn add_sources(&mut self, num: usize) -> &mut Self {
        for _ in 0..num {
            self.add_source();
        }
        self
    }

    /// Adds a new lib with the number of lib files
    pub fn add_lib(&mut self, num_files: usize) -> &mut Self {
        let lib_id = self.next_lib_id();
        let lib_name = self.name_strategy.new_lib_name(lib_id);
        let offset = self.files.len();
        for _ in 0..num_files {
            let id = self.next_file_id();
            let name = self.name_strategy.new_lib_file_name(id);
            self.files.push(MockFile {
                id,
                name,
                imports: Default::default(),
                lib_id: Some(lib_id),
            });
        }
        self.libraries.push(MockLib { name: lib_name, id: lib_id, num_files, offset });
        self
    }

    /// Populates the imports of the project
    pub fn populate_imports(&mut self, settings: &MockProjectSettings) -> &mut Self {
        let mut rng = rand::thread_rng();

        // populate imports
        for id in 0..self.files.len() {
            let imports = if let Some(lib) = self.files[id].lib_id {
                let num_imports = rng
                    .gen_range(settings.min_imports..=settings.max_imports)
                    .min(self.libraries[lib].num_files.saturating_sub(1));
                self.unique_imports_for_lib(&mut rng, lib, id, num_imports)
            } else {
                let num_imports = rng
                    .gen_range(settings.min_imports..=settings.max_imports)
                    .min(self.files.len().saturating_sub(1));
                self.unique_imports_for_source(&mut rng, id, num_imports)
            };

            self.files[id].imports = imports;
        }
        self
    }

    fn get_import(&self, id: usize) -> MockImport {
        if let Some(lib) = self.files[id].lib_id {
            MockImport::External(lib, id)
        } else {
            MockImport::Internal(id)
        }
    }

    /// All file ids
    pub fn file_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.files.iter().map(|f| f.id)
    }

    /// All ids of internal files
    pub fn internal_file_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.files.iter().filter(|f| !f.is_external()).map(|f| f.id)
    }

    /// All ids of external files
    pub fn external_file_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.files.iter().filter(|f| f.is_external()).map(|f| f.id)
    }

    /// generates exactly `num` unique imports in the range of all files
    ///
    /// # Panics
    ///
    /// if `num` can't be satisfied because the range is too narrow
    fn unique_imports_for_source<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        id: usize,
        num: usize,
    ) -> BTreeSet<MockImport> {
        assert!(self.files.len() > num);
        let mut imports: Vec<_> = (0..self.files.len()).collect();
        imports.shuffle(rng);
        imports.into_iter().filter(|i| *i != id).map(|id| self.get_import(id)).take(num).collect()
    }

    /// generates exactly `num` unique imports in the range of a lib's files
    ///
    /// # Panics
    ///
    /// if `num` can't be satisfied because the range is too narrow
    fn unique_imports_for_lib<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        lib_id: usize,
        id: usize,
        num: usize,
    ) -> BTreeSet<MockImport> {
        let lib = &self.libraries[lib_id];
        assert!(lib.num_files > num);
        let mut imports: Vec<_> = (lib.offset..(lib.offset + lib.len())).collect();
        imports.shuffle(rng);
        imports.into_iter().filter(|i| *i != id).map(|id| self.get_import(id)).take(num).collect()
    }
}

/// Used to determine the names for elements
trait NamingStrategy {
    /// Return a new name for the given source file id
    fn new_source_file_name(&mut self, id: usize) -> String;

    /// Return a new name for the given source file id
    fn new_lib_file_name(&mut self, id: usize) -> String;

    /// Return a new name for the given lib id
    fn new_lib_name(&mut self, id: usize) -> String;
}

/// A primitive naming that simply uses ids to create unique names
#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleNamingStrategy {
    _priv: (),
}

impl NamingStrategy for SimpleNamingStrategy {
    fn new_source_file_name(&mut self, id: usize) -> String {
        format!("SourceFile{}", id)
    }

    fn new_lib_file_name(&mut self, id: usize) -> String {
        format!("LibFile{}", id)
    }

    fn new_lib_name(&mut self, id: usize) -> String {
        format!("Lib{}", id)
    }
}

/// Skeleton of a mock source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockFile {
    /// internal id of this file
    pub id: usize,
    /// The source name of this file
    pub name: String,
    /// all the imported files
    pub imports: BTreeSet<MockImport>,
    /// lib id if this file is part of a lib
    pub lib_id: Option<usize>,
}

impl MockFile {
    /// Returns `true` if this file is part of an external lib
    pub fn is_external(&self) -> bool {
        self.lib_id.is_some()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum MockImport {
    /// Import from the same project
    Internal(usize),
    /// external library import
    /// (`lib id`, `file id`)
    External(usize, usize),
}

/// Container of a mock lib
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockLib {
    /// name of the lib, like `ds-test`
    pub name: String,
    /// internal id of this lib
    pub id: usize,
    /// offset in the total set of files
    pub offset: usize,
    /// number of files included in this lib
    pub num_files: usize,
}

impl MockLib {
    pub fn len(&self) -> usize {
        self.num_files
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Settings to use when generate a mock project
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MockProjectSettings {
    /// number of source files to generate
    pub num_sources: usize,
    /// number of libraries to use
    pub num_libs: usize,
    /// how many lib files to generate per lib
    pub num_lib_files: usize,
    /// min amount of import statements a file can use
    pub min_imports: usize,
    /// max amount of import statements a file can use
    pub max_imports: usize,
}

impl MockProjectSettings {
    /// Generates a new instance with random settings within an arbitrary range
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        // arbitrary thresholds
        MockProjectSettings {
            num_sources: rng.gen_range(2..25),
            num_libs: rng.gen_range(0..5),
            num_lib_files: rng.gen_range(1..10),
            min_imports: rng.gen_range(0..3),
            max_imports: rng.gen_range(4..10),
        }
    }

    /// Generates settings for a large project
    pub fn large() -> Self {
        // arbitrary thresholds
        MockProjectSettings {
            num_sources: 35,
            num_libs: 4,
            num_lib_files: 15,
            min_imports: 3,
            max_imports: 12,
        }
    }
}

impl Default for MockProjectSettings {
    fn default() -> Self {
        // these are arbitrary
        Self { num_sources: 20, num_libs: 2, num_lib_files: 10, min_imports: 0, max_imports: 5 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_generate_mock_project() {
        let _ = MockProjectGenerator::random();
    }
}
