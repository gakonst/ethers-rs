//! This is a helper module used to pass the pre-hashed message for signing to the
//! `sign_digest` methods of K256.
use crate::types::H256;
use elliptic_curve::consts::U64;
use k256::ecdsa::signature::digest::{
    generic_array::GenericArray, BlockInput, Digest, FixedOutput, Output, Reset, Update,
};

pub type Sha256Proxy = ProxyDigest<sha2::Sha256>;

#[derive(Clone)]
pub enum ProxyDigest<D: Digest> {
    Proxy(Output<D>),
    Digest(D),
}

impl<D: Digest + Clone> From<H256> for ProxyDigest<D>
where
    GenericArray<u8, <D as Digest>::OutputSize>: Copy,
{
    fn from(src: H256) -> Self {
        ProxyDigest::Proxy(*GenericArray::from_slice(src.as_bytes()))
    }
}

impl<D: Digest> Default for ProxyDigest<D> {
    fn default() -> Self {
        ProxyDigest::Digest(D::new())
    }
}

impl<D: Digest> Update for ProxyDigest<D> {
    // we update only if we are digest
    fn update(&mut self, data: impl AsRef<[u8]>) {
        match self {
            ProxyDigest::Digest(ref mut d) => {
                d.update(data);
            }
            ProxyDigest::Proxy(..) => {
                unreachable!("can not update if we are proxy");
            }
        }
    }

    // we chain only if we are digest
    fn chain(self, data: impl AsRef<[u8]>) -> Self {
        match self {
            ProxyDigest::Digest(d) => ProxyDigest::Digest(d.chain(data)),
            ProxyDigest::Proxy(..) => {
                unreachable!("can not update if we are proxy");
            }
        }
    }
}

impl<D: Digest> Reset for ProxyDigest<D> {
    // make new one
    fn reset(&mut self) {
        *self = Self::default();
    }
}

// Use Sha256 with 512 bit blocks
impl<D: Digest> BlockInput for ProxyDigest<D> {
    type BlockSize = U64;
}

impl<D: Digest> FixedOutput for ProxyDigest<D> {
    // we default to the output of the original digest
    type OutputSize = D::OutputSize;

    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        match self {
            ProxyDigest::Digest(d) => {
                *out = d.finalize();
            }
            ProxyDigest::Proxy(p) => {
                *out = p;
            }
        }
    }

    fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        let s = std::mem::take(self);
        s.finalize_into(out);
    }
}
