//! Setup utilities to start necessary infrastructure

use crate::utils::solc::{CompiledContract, SolcError};
use crate::utils::{Ganache, GanacheInstance, Geth, GethInstance, Solc};
use std::collections::HashMap;

/// Builds the contracts and returns a hashmap for each named contract
///
/// Same as [crate::utils::Solc::build] but async
pub async fn compile(solc: Solc) -> Result<HashMap<String, CompiledContract>, SolcError> {
    tokio::task::spawn_blocking(|| solc.build()).await.unwrap()
}

/// Launches a [crate::utils::GanacheInstance]
///
/// Same as [crate::utils::Ganache::spawn] but async
pub async fn launch_ganache(ganache: Ganache) -> GanacheInstance {
    tokio::task::spawn_blocking(|| ganache.spawn())
        .await
        .unwrap()
}

/// Compiles the contracts and launches a [crate::utils::GanacheInstance]
///
/// Same as [crate::utils::setup::compile] and [crate::utils::setup::launch_ganache]
pub async fn compile_and_launch_ganache(
    solc: Solc,
    ganache: Ganache,
) -> Result<(HashMap<String, CompiledContract>, GanacheInstance), SolcError> {
    let solc_fut = compile(solc);
    let ganache_fut = launch_ganache(ganache);
    let (solc, ganache) = futures_util::join!(solc_fut, ganache_fut);
    solc.map(|solc| (solc, ganache))
}

/// Launches a [crate::utils::GethInstance]
///
/// Same as [crate::utils::Geth::spawn] but async
pub async fn launch_geth(geth: Geth) -> GethInstance {
    tokio::task::spawn_blocking(|| geth.spawn()).await.unwrap()
}

/// Compiles the contracts and launches a [crate::utils::GethInstance]
///
/// Same as [crate::utils::setup::compile] and [crate::utils::setup::launch_geth]
pub async fn compile_and_launch_geth(
    solc: Solc,
    geth: Geth,
) -> Result<(HashMap<String, CompiledContract>, GethInstance), SolcError> {
    let solc_fut = compile(solc);
    let geth_fut = launch_geth(geth);
    let (solc, geth) = futures_util::join!(solc_fut, geth_fut);
    solc.map(|solc| (solc, geth))
}
