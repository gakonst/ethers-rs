//! Test cases to validate the codec
use ethers_contract::{AbiDecode, AbiEncode};
use ethers_core::abi::{AbiArrayType, TokenizableItem};
use ethers_core::rand::distributions::Alphanumeric;
use ethers_core::rand::{thread_rng, Rng};
use ethers_core::{
    rand::{
        distributions::{Distribution, Standard},
        random,
    },
    types::*,
};
use std::fmt::Debug;

fn assert_codec<T>(val: T)
where
    T: AbiDecode + AbiEncode + Clone + PartialEq + Debug,
{
    let encoded = val.clone().encode();
    assert_eq!(val, T::decode(encoded).unwrap());
}

macro_rules! roundtrip_alloc {
    ($val:expr) => {
        assert_codec($val);
        assert_codec(($val, $val));
        assert_codec(std::iter::repeat_with(|| $val).take(10).collect::<Vec<_>>());
    };
}

macro_rules! roundtrip_all {
    ($val:expr) => {
        roundtrip_alloc!($val);
        assert_codec([$val; 10]);
    };
}

fn test_codec_rng<T>()
where
    Standard: Distribution<T>,
    T: AbiDecode + AbiEncode + Copy + PartialEq + Debug + AbiArrayType + TokenizableItem,
{
    roundtrip_all!(random::<T>());
}

#[test]
fn address_codec() {
    test_codec_rng::<Address>();
}

#[test]
fn uint_codec() {
    test_codec_rng::<u16>();
    test_codec_rng::<u32>();
    test_codec_rng::<u64>();
    test_codec_rng::<u128>();

    test_codec_rng::<i8>();
    test_codec_rng::<i16>();
    test_codec_rng::<i32>();
    test_codec_rng::<i64>();
    test_codec_rng::<i128>();
}

#[test]
fn u8_codec() {
    // assert_codec(random::<u8>());
    // assert_codec((random::<u8>(), random::<u8>()));
    // assert_codec(
    //     std::iter::repeat_with(|| random::<u8>())
    //         .take(10)
    //         .collect::<Vec<_>>(),
    // );
    // assert_codec([random::<u8>(); 10]);
}

#[test]
fn string_codec() {
    roundtrip_alloc! { thread_rng()
    .sample_iter(&Alphanumeric)
    .take(30)
    .map(char::from)
    .collect::<String>()
    };
}
