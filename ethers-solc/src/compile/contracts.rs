use crate::artifacts::{CompactContractRef, Contract, FileToContractsMap};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

    /// Finds the _first_ contract with the given name
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap().output();
    /// let contract = output.find("Greeter").unwrap();
    /// # }
    /// ```
    pub fn find(&self, contract: impl AsRef<str>) -> Option<CompactContractRef> {
        let contract_name = contract.as_ref();
        self.contracts().find_map(|(name, contract)| {
            (name == contract_name).then(|| CompactContractRef::from(contract))
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
    /// let contract = contracts.remove("Greeter").unwrap();
    /// # }
    /// ```
    pub fn remove(&mut self, contract: impl AsRef<str>) -> Option<Contract> {
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

    /// Given the contract file's path and the contract's name, tries to return the contract's
    /// bytecode, runtime bytecode, and abi
    pub fn get(&self, path: &str, contract: &str) -> Option<CompactContractRef> {
        self.0
            .get(path)
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

    /// Returns an iterator over all contracts and their source names.
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use ethers_solc::{ artifacts::*, Artifact };
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
