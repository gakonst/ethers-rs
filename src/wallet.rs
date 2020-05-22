use crate::{primitives::Signature, providers::Provider};

pub struct Signer<'a> {
    provider: Option<&'a Provider>,
}

impl<'a> Signer<'a> {
    pub fn random() -> Self {
        Signer { provider: None }
    }

    pub fn connect(mut self, provider: &'a Provider) -> Self {
        self.provider = Some(provider);
        self
    }
}

trait SignerC {
    /// Connects to a provider
    fn connect<'a>(self, provider: &'a Provider) -> Self;

    fn sign_message(message: &[u8]) -> Signature;
}
