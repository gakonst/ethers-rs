use ethers_core::types::{Chain, H160};

/// The Multicall3 contract address that is deployed in [`MULTICALL_SUPPORTED_CHAIN_IDS`]:
/// [`0xcA11bde05977b3631167028862bE2a173976CA11`](https://etherscan.io/address/0xcA11bde05977b3631167028862bE2a173976CA11)
pub const MULTICALL_ADDRESS: H160 = H160([
    0xca, 0x11, 0xbd, 0xe0, 0x59, 0x77, 0xb3, 0x63, 0x11, 0x67, 0x02, 0x88, 0x62, 0xbe, 0x2a, 0x17,
    0x39, 0x76, 0xca, 0x11,
]);

/// The chain IDs that [`MULTICALL_ADDRESS`] has been deployed to.
///
/// Taken from: <https://github.com/mds1/multicall#multicall3-contract-addresses>
pub const MULTICALL_SUPPORTED_CHAIN_IDS: &[u64] = {
    use Chain::*;
    &[
        Mainnet as u64,                  // Mainnet
        Kovan as u64,                    // Kovan
        Rinkeby as u64,                  // Rinkeby
        Goerli as u64,                   // Görli
        Ropsten as u64,                  // Ropsten
        Sepolia as u64,                  // Sepolia
        Optimism as u64,                 // Optimism
        OptimismKovan as u64,            // Optimism Kovan
        OptimismGoerli as u64,           // Optimism Görli
        Arbitrum as u64,                 // Arbitrum
        ArbitrumNova as u64,             // Arbitrum Nova
        ArbitrumGoerli as u64,           // Arbitrum Görli
        ArbitrumTestnet as u64,          // Arbitrum Rinkeby
        Polygon as u64,                  // Polygon
        PolygonMumbai as u64,            // Polygon Mumbai
        XDai as u64,                     // Gnosis Chain
        Avalanche as u64,                // Avalanche
        AvalancheFuji as u64,            // Avalanche Fuji
        FantomTestnet as u64,            // Fantom Testnet
        Fantom as u64,                   // Fantom Opera
        BinanceSmartChain as u64,        // BNB Smart Chain
        BinanceSmartChainTestnet as u64, // BNB Smart Chain Testnet
        Moonbeam as u64,                 // Moonbeam
        Moonriver as u64,                // Moonriver
        Moonbase as u64,                 // Moonbase
        1666600000,                      // Harmony0
        1666600001,                      // Harmony1
        1666600002,                      // Harmony2
        1666600003,                      // Harmony3
        Cronos as u64,                   // Cronos
        122,                             // Fuse
        14,                              // Flare Mainnet
        19,                              // Songbird Canary Network
        16,                              // Coston Testnet
        114,                             // Coston2 Testnet
        288,                             // Boba
        Aurora as u64,                   // Aurora
        592,                             // Astar
        66,                              // OKC
        128,                             // Heco Chain
        1088,                            // Metis
        Rsk as u64,                      // Rsk
        31,                              // Rsk Testnet
        Evmos as u64,                    // Evmos
        EvmosTestnet as u64,             // Evmos Testnet
        Oasis as u64,                    // Oasis
        42261,                           // Oasis Emerald ParaTime Testnet
        42262,                           // Oasis Emerald ParaTime
        Celo as u64,                     // Celo
        CeloAlfajores as u64,            // Celo Alfajores Testnet
        71402,                           // Godwoken
        71401,                           // Godwoken Testnet
        8217,                            // Klaytn
        2001,                            // Milkomeda
        321,                             // KCC
        106,                             // Velas
        40,                              // Telos
    ]
};
