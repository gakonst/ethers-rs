use crate::{
    artifacts::{
        contract::{CompactContractRef, Contract},
        FileToContractsMap,
    },
    ArtifactFiles, ArtifactOutput,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::Deref, path::Path};

/// file -> [(contract name  -> Contract + solc version)]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VersionedContracts(pub FileToContractsMap<Vec<VersionedContract>>);

impl VersionedContracts {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over all files
    pub fn files(&self) -> impl Iterator<Item = &String> + '_ {
        self.0.keys()
    }

    /// Returns all the artifact files mapped with their contracts
    pub(crate) fn artifact_files<T: ArtifactOutput + ?Sized>(&self) -> ArtifactFiles {
        let mut output_files = ArtifactFiles::with_capacity(self.len());
        for (file, contracts) in self.iter() {
            for (name, versioned_contracts) in contracts {
                for contract in versioned_contracts {
                    let output = if versioned_contracts.len() > 1 {
                        T::output_file_versioned(file, name, &contract.version)
                    } else {
                        T::output_file(file, name)
                    };
                    let contract = (file.as_str(), name.as_str(), contract);
                    output_files.entry(output).or_default().push(contract);
                }
            }
        }

        output_files
    }

    /// Finds the _first_ contract with the given name
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap().output();
    /// let contract = output.find_first("Greeter").unwrap();
    /// # }
    /// ```
    pub fn find_first(&self, contract: impl AsRef<str>) -> Option<CompactContractRef> {
        let contract_name = contract.as_ref();
        self.contracts().find_map(|(name, contract)| {
            (name == contract_name).then(|| CompactContractRef::from(contract))
        })
    }

    /// Finds the contract with matching path and name
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap().output();
    /// let contract = output.contracts.find("src/Greeter.sol", "Greeter").unwrap();
    /// # }
    /// ```
    pub fn find(
        &self,
        path: impl AsRef<str>,
        contract: impl AsRef<str>,
    ) -> Option<CompactContractRef> {
        let contract_path = path.as_ref();
        let contract_name = contract.as_ref();
        self.contracts_with_files().find_map(|(path, name, contract)| {
            (path == contract_path && name == contract_name)
                .then(|| CompactContractRef::from(contract))
        })
    }

    /// Removes the _first_ contract with the given name from the set
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let (_, mut contracts) = project.compile().unwrap().output().split();
    /// let contract = contracts.remove_first("Greeter").unwrap();
    /// # }
    /// ```
    pub fn remove_first(&mut self, contract: impl AsRef<str>) -> Option<Contract> {
        let contract_name = contract.as_ref();
        self.0.values_mut().find_map(|all_contracts| {
            let mut contract = None;
            if let Some((c, mut contracts)) = all_contracts.remove_entry(contract_name) {
                if !contracts.is_empty() {
                    contract = Some(contracts.remove(0).contract);
                }
                if !contracts.is_empty() {
                    all_contracts.insert(c, contracts);
                }
            }
            contract
        })
    }

    ///  Removes the contract with matching path and name
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let (_, mut contracts) = project.compile().unwrap().output().split();
    /// let contract = contracts.remove("src/Greeter.sol", "Greeter").unwrap();
    /// # }
    /// ```
    pub fn remove(&mut self, path: impl AsRef<str>, contract: impl AsRef<str>) -> Option<Contract> {
        let contract_name = contract.as_ref();
        let (key, mut all_contracts) = self.0.remove_entry(path.as_ref())?;
        let mut contract = None;
        if let Some((c, mut contracts)) = all_contracts.remove_entry(contract_name) {
            if !contracts.is_empty() {
                contract = Some(contracts.remove(0).contract);
            }
            if !contracts.is_empty() {
                all_contracts.insert(c, contracts);
            }
        }

        if !all_contracts.is_empty() {
            self.0.insert(key, all_contracts);
        }
        contract
    }

    /// Given the contract file's path and the contract's name, tries to return the contract's
    /// bytecode, runtime bytecode, and abi
    pub fn get(
        &self,
        path: impl AsRef<str>,
        contract: impl AsRef<str>,
    ) -> Option<CompactContractRef> {
        let contract = contract.as_ref();
        self.0
            .get(path.as_ref())
            .and_then(|contracts| {
                contracts.get(contract).and_then(|c| c.get(0).map(|c| &c.contract))
            })
            .map(CompactContractRef::from)
    }

    /// Iterate over all contracts and their names
    pub fn contracts(&self) -> impl Iterator<Item = (&String, &Contract)> {
        self.0
            .values()
            .flat_map(|c| c.iter().flat_map(|(name, c)| c.iter().map(move |c| (name, &c.contract))))
    }

    /// Returns an iterator over (`file`, `name`, `Contract`)
    pub fn contracts_with_files(&self) -> impl Iterator<Item = (&String, &String, &Contract)> {
        self.0.iter().flat_map(|(file, contracts)| {
            contracts
                .iter()
                .flat_map(move |(name, c)| c.iter().map(move |c| (file, name, &c.contract)))
        })
    }

    /// Returns an iterator over (`file`, `name`, `Contract`, `Version`)
    pub fn contracts_with_files_and_version(
        &self,
    ) -> impl Iterator<Item = (&String, &String, &Contract, &Version)> {
        self.0.iter().flat_map(|(file, contracts)| {
            contracts.iter().flat_map(move |(name, c)| {
                c.iter().map(move |c| (file, name, &c.contract, &c.version))
            })
        })
    }

    /// Returns an iterator over all contracts and their source names.
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use ethers_solc::{ artifacts::*, Artifact };
    /// use ethers_solc::artifacts::contract::CompactContractSome;
    /// # fn demo(contracts: OutputContracts) {
    /// let contracts: BTreeMap<String, CompactContractSome> = contracts
    ///     .into_contracts()
    ///     .map(|(k, c)| (k, c.into_compact_contract().unwrap()))
    ///     .collect();
    /// # }
    /// ```
    pub fn into_contracts(self) -> impl Iterator<Item = (String, Contract)> {
        self.0.into_values().flat_map(|c| {
            c.into_iter()
                .flat_map(|(name, c)| c.into_iter().map(move |c| (name.clone(), c.contract)))
        })
    }

    /// Returns an iterator over (`file`, `name`, `Contract`)
    pub fn into_contracts_with_files(self) -> impl Iterator<Item = (String, String, Contract)> {
        self.0.into_iter().flat_map(|(file, contracts)| {
            contracts.into_iter().flat_map(move |(name, c)| {
                let file = file.clone();
                c.into_iter().map(move |c| (file.clone(), name.clone(), c.contract))
            })
        })
    }

    /// Returns an iterator over (`file`, `name`, `Contract`, `Version`)
    pub fn into_contracts_with_files_and_version(
        self,
    ) -> impl Iterator<Item = (String, String, Contract, Version)> {
        self.0.into_iter().flat_map(|(file, contracts)| {
            contracts.into_iter().flat_map(move |(name, c)| {
                let file = file.clone();
                c.into_iter().map(move |c| (file.clone(), name.clone(), c.contract, c.version))
            })
        })
    }

    /// Sets the contract's file paths to `root` adjoined to `self.file`.
    pub fn join_all(&mut self, root: impl AsRef<Path>) -> &mut Self {
        let root = root.as_ref();
        self.0 = std::mem::take(&mut self.0)
            .into_iter()
            .map(|(contract_path, contracts)| {
                (format!("{}", root.join(contract_path).display()), contracts)
            })
            .collect();
        self
    }

    /// Removes `base` from all contract paths
    pub fn strip_prefix_all(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();
        self.0 = std::mem::take(&mut self.0)
            .into_iter()
            .map(|(contract_path, contracts)| {
                let p = Path::new(&contract_path);
                (
                    p.strip_prefix(base)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or(contract_path),
                    contracts,
                )
            })
            .collect();
        self
    }
}

impl AsRef<FileToContractsMap<Vec<VersionedContract>>> for VersionedContracts {
    fn as_ref(&self) -> &FileToContractsMap<Vec<VersionedContract>> {
        &self.0
    }
}

impl AsMut<FileToContractsMap<Vec<VersionedContract>>> for VersionedContracts {
    fn as_mut(&mut self) -> &mut FileToContractsMap<Vec<VersionedContract>> {
        &mut self.0
    }
}

impl Deref for VersionedContracts {
    type Target = FileToContractsMap<Vec<VersionedContract>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for VersionedContracts {
    type Item = (String, BTreeMap<String, Vec<VersionedContract>>);
    type IntoIter =
        std::collections::btree_map::IntoIter<String, BTreeMap<String, Vec<VersionedContract>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A contract and the compiler version used to compile it
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VersionedContract {
    pub contract: Contract,
    pub version: Version,
}
