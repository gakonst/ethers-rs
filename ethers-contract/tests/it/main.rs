#![allow(unused)]

mod abigen;
pub(crate) mod common;
#[cfg(feature = "abigen")]
mod console;
#[cfg(feature = "abigen")]
mod contract;
mod contract_call;

fn main() {}
