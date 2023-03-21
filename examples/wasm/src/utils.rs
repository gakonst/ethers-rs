use ethers::{
    prelude::{LocalWallet, MnemonicBuilder},
    signers::coins_bip39::English,
};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// The mnemonic phrase used by ganache
pub const PHRASE: &str =
    "stuff inherit faith park genre spread huge knee ecology private marble supreme";

pub fn key(index: u32) -> LocalWallet {
    MnemonicBuilder::<English>::default().phrase(PHRASE).index(index).unwrap().build().unwrap()
}
