use super::show::ShowKeyCmd;
use crate::application::APP;
use abscissa_core::{Application, Command, Clap, Runnable};
use k256::pkcs8::ToPrivateKey;
use signatory::FsKeyStore;
use std::path;
use ethers_core::
    types;

/// Import Key
#[derive(Command, Debug, Default, Clap)]
pub struct ImportKeyCmd {
    pub args: Vec<String>,

    #[clap(short, long)]
    pub overwrite: bool,
}

// Entry point for `keys eth import [name] (bip39-mnemonic)`
// - [name] required; key name
// - (bip39-mnemonic) optional; when absent the user will be prompted to enter it
impl Runnable for ImportKeyCmd {
    fn run(&self) {
        let config = APP.config();
        let keystore = path::Path::new("/tmp/keystore");
        let keystore = FsKeyStore::create_or_open(keystore).expect("Could not open keystore");

        let name = self.args.get(0).expect("name is required");
        let name = name.parse().expect("Could not parse name");
        if let Ok(_info) = keystore.info(&name) {
            if !self.overwrite {
                eprintln!("Key already exists, exiting.");
                return;
            }
        }

        let mnemonic = match self.args.get(1) {
            Some(mnemonic) => mnemonic.clone(),
            None => rpassword::read_password_from_tty(Some("> Enter your bip39 mnemonic:\n"))
                .expect("Could not read mnemonic"),
        };

        let key = match bip32::Mnemonic::new(mnemonic.trim(), Default::default()) {
            Ok(mnemonic) => {
                let seed = mnemonic.to_seed("");

                let path = "m/44'/118'/0'/0/0".trim();
                let path = path
                    .parse::<bip32::DerivationPath>()
                    .expect("Could not parse derivation path");

                let key = bip32::XPrv::derive_from_path(seed, &path).expect("Could not derive key");
                let key = k256::SecretKey::from(key.private_key());
                key.to_pkcs8_der()
                    .expect("Could not PKCS8 encod private key")
            }
            Err(_) => {
                let key = rpassword::read_password_from_tty(Some("> Enter your private-key:\n"))
                    .expect("Could not read private-key");

                let key: types::H256 = key.parse().expect("Could not parse private-key");

                let key = k256::SecretKey::from_bytes(key).expect("Could not make private key");
                key.to_pkcs8_der()
                    .expect("Could not PKCS8 encode private key")
            }
        };

        keystore.store(&name, &key).expect("Could not store key");

        let args = vec![name.to_string()];
        let show_cmd = ShowKeyCmd { args };
        show_cmd.run();
    }
}