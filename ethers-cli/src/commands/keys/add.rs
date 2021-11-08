use crate::application::APP;
use abscissa_core::{Application, Command, Clap, Runnable};
use bip32;
use k256::pkcs8::ToPrivateKey;
use rand_core::OsRng;
use signatory::FsKeyStore;
use std::path;

/// Create a new Key
#[derive(Command, Debug, Default, Clap)]
pub struct AddKeyCmd {
    pub args: Vec<String>,

    #[clap(short, long)]
    pub overwrite: bool,
}

// `contract monitor keys add [name]`
// - [name] required; key name
impl Runnable for AddKeyCmd {
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

        let mnemonic = bip32::Mnemonic::random(&mut OsRng, Default::default());
        eprintln!("**Important** record this bip39-mnemonic in a safe place:");
        println!("{}", mnemonic.phrase());

        let seed = mnemonic.to_seed("");

        let path = "m/44'/118'/0'/0/0".trim();
        let path = path
            .parse::<bip32::DerivationPath>()
            .expect("Could not parse derivation path");

        let key = bip32::XPrv::derive_from_path(seed, &path).expect("Could not derive key");
        let key = k256::SecretKey::from(key.private_key());
        let key = key
            .to_pkcs8_der()
            .expect("Could not PKCS8 encod private key");

        keystore.store(&name, &key).expect("Could not store key");
    }
}