//! Specific helper functions for creating/loading a mnemonic private key following BIP-39
//! specifications
use crate::{Wallet, WalletError};

use coins_bip32::path::DerivationPath;
use coins_bip39::{Mnemonic, Wordlist};
use ethers_core::{
    k256::ecdsa::SigningKey,
    types::PathOrString,
    utils::{secret_key_to_address, to_checksum},
};
use rand::Rng;
use std::{fs::File, io::Write, marker::PhantomData, path::PathBuf, str::FromStr};
use thiserror::Error;

const DEFAULT_DERIVATION_PATH_PREFIX: &str = "m/44'/60'/0'/0/";

/// Represents a structure that can resolve into a `Wallet<SigningKey>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MnemonicBuilder<W: Wordlist> {
    /// The mnemonic phrase can be supplied to the builder as a string or a path to the file whose
    /// contents are the phrase. A builder that has a valid phrase should `build` the wallet.
    phrase: Option<PathOrString>,
    /// The mnemonic builder can also be asked to generate a new random wallet by providing the
    /// number of words in the phrase. By default this is set to 12.
    word_count: usize,
    /// The derivation path at which the extended private key child will be derived at. By default
    /// the mnemonic builder uses the path: "m/44'/60'/0'/0/0".
    derivation_path: DerivationPath,
    /// Optional password for the mnemonic phrase.
    password: Option<String>,
    /// Optional field that if enabled, writes the mnemonic phrase to disk storage at the provided
    /// path.
    write_to: Option<PathBuf>,
    /// PhantomData
    _wordlist: PhantomData<W>,
}

/// Error produced by the mnemonic wallet module
#[derive(Error, Debug)]
pub enum MnemonicBuilderError {
    /// Error suggests that a phrase (path or words) was expected but not found
    #[error("Expected phrase not found")]
    ExpectedPhraseNotFound,
    /// Error suggests that a phrase (path or words) was not expected but found
    #[error("Unexpected phrase found")]
    UnexpectedPhraseFound,
}

impl<W: Wordlist> Default for MnemonicBuilder<W> {
    fn default() -> Self {
        Self {
            phrase: None,
            word_count: 12usize,
            derivation_path: DerivationPath::from_str(&format!(
                "{}{}",
                DEFAULT_DERIVATION_PATH_PREFIX, 0
            ))
            .expect("should parse the default derivation path"),
            password: None,
            write_to: None,
            _wordlist: PhantomData,
        }
    }
}

impl<W: Wordlist> MnemonicBuilder<W> {
    /// Sets the phrase in the mnemonic builder. The phrase can either be a string or a path to
    /// the file that contains the phrase. Once a phrase is provided, the key will be generated
    /// deterministically by calling the `build` method.
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_signers::{MnemonicBuilder, coins_bip39::English};
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let wallet = MnemonicBuilder::<English>::default()
    ///     .phrase("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")
    ///     .build()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn phrase<P: Into<PathOrString>>(mut self, phrase: P) -> Self {
        self.phrase = Some(phrase.into());
        self
    }

    /// Sets the word count of a mnemonic phrase to be generated at random. If the `phrase` field
    /// is set, then `word_count` will be ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_signers::{MnemonicBuilder, coins_bip39::English};
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let mut rng = rand::thread_rng();
    /// let wallet = MnemonicBuilder::<English>::default()
    ///     .word_count(24)
    ///     .build_random(&mut rng)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn word_count(mut self, count: usize) -> Self {
        self.word_count = count;
        self
    }

    /// Sets the derivation path of the child key to be derived. The derivation path is calculated
    /// using the default derivation path prefix used in Ethereum, i.e. "m/44'/60'/0'/0/{index}".
    pub fn index<U: Into<u32>>(mut self, index: U) -> Result<Self, WalletError> {
        self.derivation_path = DerivationPath::from_str(&format!(
            "{}{}",
            DEFAULT_DERIVATION_PATH_PREFIX,
            index.into()
        ))?;
        Ok(self)
    }

    /// Sets the derivation path of the child key to be derived.
    pub fn derivation_path(mut self, path: &str) -> Result<Self, WalletError> {
        self.derivation_path = DerivationPath::from_str(path)?;
        Ok(self)
    }

    /// Sets the password used to construct the seed from the mnemonic phrase.
    #[must_use]
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    /// Sets the path to which the randomly generated phrase will be written to. This field is
    /// ignored when building a wallet from the provided mnemonic phrase.
    #[must_use]
    pub fn write_to<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.write_to = Some(path.into());
        self
    }

    /// Builds a `LocalWallet` using the parameters set in mnemonic builder. This method expects
    /// the phrase field to be set.
    pub fn build(&self) -> Result<Wallet<SigningKey>, WalletError> {
        let mnemonic = match &self.phrase {
            Some(path_or_string) => {
                let phrase = path_or_string.read()?;
                Mnemonic::<W>::new_from_phrase(&phrase)?
            }
            None => return Err(MnemonicBuilderError::ExpectedPhraseNotFound.into()),
        };
        self.mnemonic_to_wallet(&mnemonic)
    }

    /// Builds a `LocalWallet` using the parameters set in the mnemonic builder and constructing
    /// the phrase using the provided random number generator.
    pub fn build_random<R: Rng>(&self, rng: &mut R) -> Result<Wallet<SigningKey>, WalletError> {
        let mnemonic = match &self.phrase {
            None => Mnemonic::<W>::new_with_count(rng, self.word_count)?,
            _ => return Err(MnemonicBuilderError::UnexpectedPhraseFound.into()),
        };
        let wallet = self.mnemonic_to_wallet(&mnemonic)?;

        // Write the mnemonic phrase to storage if a directory has been provided.
        if let Some(dir) = &self.write_to {
            let mut file = File::create(dir.as_path().join(to_checksum(&wallet.address, None)))?;
            file.write_all(mnemonic.to_phrase().as_bytes())?;
        }

        Ok(wallet)
    }

    fn mnemonic_to_wallet(
        &self,
        mnemonic: &Mnemonic<W>,
    ) -> Result<Wallet<SigningKey>, WalletError> {
        let derived_priv_key =
            mnemonic.derive_key(&self.derivation_path, self.password.as_deref())?;
        let key: &coins_bip32::prelude::SigningKey = derived_priv_key.as_ref();
        let signer = SigningKey::from_bytes(&key.to_bytes())?;
        let address = secret_key_to_address(&signer);

        Ok(Wallet::<SigningKey> { signer, address, chain_id: 1 })
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;

    use crate::coins_bip39::English;
    use tempfile::tempdir;

    const TEST_DERIVATION_PATH: &str = "m/44'/60'/0'/2/1";

    #[tokio::test]
    async fn mnemonic_deterministic() {
        // Testcases have been taken from MyCryptoWallet
        const TESTCASES: [(&str, u32, Option<&str>, &str); 4] = [
            (
                "work man father plunge mystery proud hollow address reunion sauce theory bonus",
                0u32,
                Some("TREZOR123"),
                "0x431a00DA1D54c281AeF638A73121B3D153e0b0F6",
            ),
            (
                "inject danger program federal spice bitter term garbage coyote breeze thought funny",
                1u32,
                Some("LEDGER321"),
                "0x231a3D0a05d13FAf93078C779FeeD3752ea1350C",
            ),
            (
                "fire evolve buddy tenant talent favorite ankle stem regret myth dream fresh",
                2u32,
                None,
                "0x1D86AD5eBb2380dAdEAF52f61f4F428C485460E9",
            ),
            (
                "thumb soda tape crunch maple fresh imitate cancel order blind denial giraffe",
                3u32,
                None,
                "0xFB78b25f69A8e941036fEE2A5EeAf349D81D4ccc",
            ),
        ];
        TESTCASES.iter().for_each(|(phrase, index, password, expected_addr)| {
            let wallet = match password {
                Some(psswd) => MnemonicBuilder::<English>::default()
                    .phrase(*phrase)
                    .index(*index)
                    .unwrap()
                    .password(psswd)
                    .build()
                    .unwrap(),
                None => MnemonicBuilder::<English>::default()
                    .phrase(*phrase)
                    .index(*index)
                    .unwrap()
                    .build()
                    .unwrap(),
            };
            assert_eq!(&to_checksum(&wallet.address, None), expected_addr);
        })
    }

    #[tokio::test]
    async fn mnemonic_write_read() {
        let dir = tempdir().unwrap();

        // Construct a wallet from random mnemonic phrase and write it to the temp dir.
        let mut rng = rand::thread_rng();
        let wallet1 = MnemonicBuilder::<English>::default()
            .word_count(24)
            .derivation_path(TEST_DERIVATION_PATH)
            .unwrap()
            .write_to(dir.as_ref())
            .build_random(&mut rng)
            .unwrap();

        // Ensure that only one file has been created.
        let paths = std::fs::read_dir(dir.as_ref()).unwrap();
        assert_eq!(paths.count(), 1);

        // Use the newly created file's path to instantiate wallet.
        let phrase_path = dir.as_ref().join(to_checksum(&wallet1.address, None));
        let wallet2 = MnemonicBuilder::<English>::default()
            .phrase(phrase_path.to_str().unwrap())
            .derivation_path(TEST_DERIVATION_PATH)
            .unwrap()
            .build()
            .unwrap();

        // Ensure that both wallets belong to the same address.
        assert_eq!(wallet1.address, wallet2.address);

        dir.close().unwrap();
    }
}
