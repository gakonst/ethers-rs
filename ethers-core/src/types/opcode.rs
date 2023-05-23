use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumCount, EnumIter, EnumString, EnumVariantNames};

// opcode descriptions taken from evm.codes https://github.com/comitylabs/evm.codes/blob/bc7f102808055d88365559d40c190c5bd6d164c3/opcodes.json
// https://github.com/ethereum/go-ethereum/blob/2b1299b1c006077c56ecbad32e79fc16febe3dd6/core/vm/opcodes.go

/// An [EVM Opcode](https://evm.codes).
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsRefStr,
    Display,
    EnumString,
    EnumVariantNames,
    EnumIter,
    EnumCount,
    TryFromPrimitive,
    Serialize,
    Deserialize,
)]
#[repr(u8)]
pub enum Opcode {
    // 0x0 range - arithmetic ops.
    /// Opcode 0x0 - Halts execution
    STOP = 0x00,
    /// Opcode 0x1 - Addition operation
    ADD,
    /// Opcode 0x2 - Multiplication operation
    MUL,
    /// Opcode 0x3 - Subtraction operation
    SUB,
    /// Opcode 0x4 - Integer division operation
    DIV,
    /// Opcode 0x5 - Signed integer division operation (truncated)
    SDIV,
    /// Opcode 0x6 - Modulo remainder operation
    MOD,
    /// Opcode 0x7 - Signed modulo remainder operation
    SMOD,
    /// Opcode 0x8 - Modulo addition operation
    ADDMOD,
    /// Opcode 0x9 - Modulo multiplication operation
    MULMOD,
    /// Opcode 0xA - Exponential operation
    EXP,
    /// Opcode 0xB - Extend length of two’s complement signed integer
    SIGNEXTEND,

    // 0x0C - 0x0F are invalid

    // 0x10 range - comparison ops.
    /// Opcode 0x10 - Less-than comparison
    LT = 0x10,
    /// Opcode 0x11 - Greater-than comparison
    GT,
    /// Opcode 0x12 - Signed less-than comparison
    SLT,
    /// Opcode 0x13 - Signed greater-than comparison
    SGT,
    /// Opcode 0x14 - Equality comparison
    EQ,
    /// Opcode 0x15 - Simple not operator
    ISZERO,
    /// Opcode 0x16 - Bitwise AND operation
    AND,
    /// Opcode 0x17 - Bitwise OR operation
    OR,
    /// Opcode 0x18 - Bitwise XOR operation
    XOR,
    /// Opcode 0x19 - Bitwise NOT operation
    NOT,
    /// Opcode 0x1A - Retrieve single byte from word
    BYTE,
    /// Opcode 0x1B - Left shift operation
    SHL,
    /// Opcode 0x1C - Logical right shift operation
    SHR,
    /// Opcode 0x1D - Arithmetic (signed) right shift operation
    SAR,

    // 0x1E - 0x1F are invalid

    // 0x20 range - crypto.
    /// Opcode 0x20 - Compute Keccak-256 hash
    #[serde(alias = "KECCAK256")]
    SHA3 = 0x20,

    // 0x21 - 0x2F are invalid

    // 0x30 range - closure state.
    /// Opcode 0x30 - Get address of currently executing account
    ADDRESS = 0x30,
    /// Opcode 0x31 - Get address of currently executing account
    BALANCE,
    /// Opcode 0x32 - Get execution origination address
    ORIGIN,
    /// Opcode 0x33 - Get caller address
    CALLER,
    /// Opcode 0x34 - Get deposited value by the instruction/transaction responsible for this
    /// execution
    CALLVALUE,
    /// Opcode 0x35 - Get input data of current environment
    CALLDATALOAD,
    /// Opcode 0x36 - Get size of input data in current environment
    CALLDATASIZE,
    /// Opcode 0x37 - Copy input data in current environment to memory
    CALLDATACOPY,
    /// Opcode 0x38 - Get size of code running in current environment
    CODESIZE,
    /// Opcode 0x39 - Copy code running in current environment to memory
    CODECOPY,
    /// Opcode 0x3A - Get price of gas in current environment
    GASPRICE,
    /// Opcode 0x3B - Get size of an account’s code
    EXTCODESIZE,
    /// Opcode 0x3C - Copy an account’s code to memory
    EXTCODECOPY,
    /// Opcode 0x3D - Get size of output data from the previous call from the current environment
    RETURNDATASIZE,
    /// Opcode 0x3E - Copy output data from the previous call to memory
    RETURNDATACOPY,
    /// Opcode 0x3F - Get hash of an account’s code
    EXTCODEHASH,

    // 0x40 range - block operations.
    /// Opcode 0x40 - Get the hash of one of the 256 most recent complete blocks
    BLOCKHASH = 0x40,
    /// Opcode 0x41 - Get the block’s beneficiary address
    COINBASE,
    /// Opcode 0x42 - Get the block’s timestamp
    TIMESTAMP,
    /// Opcode 0x43 - Get the block’s number
    NUMBER,
    /// Opcode 0x44 - Get the block’s difficulty
    #[serde(alias = "PREVRANDAO", alias = "RANDOM")]
    #[strum(to_string = "DIFFICULTY", serialize = "PREVRANDAO", serialize = "RANDOM")]
    DIFFICULTY,
    /// Opcode 0x45 - Get the block’s gas limit
    GASLIMIT,
    /// Opcode 0x46 - Get the chain ID
    CHAINID,
    /// Opcode 0x47 - Get balance of currently executing account
    SELFBALANCE,
    /// Opcode 0x48 - Get the base fee
    BASEFEE,

    // 0x49 - 0x4F are invalid

    // 0x50 range - 'storage' and execution.
    /// Opcode 0x50 - Remove item from stack
    POP = 0x50,
    /// Opcode 0x51 - Load word from memory
    MLOAD,
    /// Opcode 0x52 - Save word to memory
    MSTORE,
    /// Opcode 0x53 - Save byte to memory
    MSTORE8,
    /// Opcode 0x54 - Load word from storage
    SLOAD,
    /// Opcode 0x55 - Save word to storage
    SSTORE,
    /// Opcode 0x56 - Alter the program counter
    JUMP,
    /// Opcode 0x57 - Conditionally alter the program counter
    JUMPI,
    /// Opcode 0x58 - Get the value of the program counter prior to the increment corresponding to
    /// this instruction
    PC,
    /// Opcode 0x59 - Get the size of active memory in bytes
    MSIZE,
    /// Opcode 0x5A - Get the amount of available gas, including the corresponding reduction for
    /// the cost of this instruction
    GAS,
    /// Opcode 0x5B - Mark a valid destination for jumps
    JUMPDEST,

    // 0x5C - 0x5E are invalid

    // 0x5F range - pushes.
    /// Opcode 0x5F - Place the constant value 0 on stack
    PUSH0 = 0x5f,
    /// Opcode 0x60 - Place 1 byte item on stack
    PUSH1 = 0x60,
    /// Opcode 0x61 - Place 2 byte item on stack
    PUSH2,
    /// Opcode 0x62 - Place 3 byte item on stack
    PUSH3,
    /// Opcode 0x63 - Place 4 byte item on stack
    PUSH4,
    /// Opcode 0x64 - Place 5 byte item on stack
    PUSH5,
    /// Opcode 0x65 - Place 6 byte item on stack
    PUSH6,
    /// Opcode 0x66 - Place 7 byte item on stack
    PUSH7,
    /// Opcode 0x67 - Place 8 byte item on stack
    PUSH8,
    /// Opcode 0x68 - Place 9 byte item on stack
    PUSH9,
    /// Opcode 0x69 - Place 10 byte item on stack
    PUSH10,
    /// Opcode 0x6A - Place 11 byte item on stack
    PUSH11,
    /// Opcode 0x6B - Place 12 byte item on stack
    PUSH12,
    /// Opcode 0x6C - Place 13 byte item on stack
    PUSH13,
    /// Opcode 0x6D - Place 14 byte item on stack
    PUSH14,
    /// Opcode 0x6E - Place 15 byte item on stack
    PUSH15,
    /// Opcode 0x6F - Place 16 byte item on stack
    PUSH16,
    /// Opcode 0x70 - Place 17 byte item on stack
    PUSH17,
    /// Opcode 0x71 - Place 18 byte item on stack
    PUSH18,
    /// Opcode 0x72 - Place 19 byte item on stack
    PUSH19,
    /// Opcode 0x73 - Place 20 byte item on stack
    PUSH20,
    /// Opcode 0x74 - Place 21 byte item on stack
    PUSH21,
    /// Opcode 0x75 - Place 22 byte item on stack
    PUSH22,
    /// Opcode 0x76 - Place 23 byte item on stack
    PUSH23,
    /// Opcode 0x77 - Place 24 byte item on stack
    PUSH24,
    /// Opcode 0x78 - Place 25 byte item on stack
    PUSH25,
    /// Opcode 0x79 - Place 26 byte item on stack
    PUSH26,
    /// Opcode 0x7A - Place 27 byte item on stack
    PUSH27,
    /// Opcode 0x7B - Place 28 byte item on stack
    PUSH28,
    /// Opcode 0x7C - Place 29 byte item on stack
    PUSH29,
    /// Opcode 0x7D - Place 30 byte item on stack
    PUSH30,
    /// Opcode 0x7E - Place 31 byte item on stack
    PUSH31,
    /// Opcode 0x7F - Place 32 byte item on stack
    PUSH32,

    // 0x80 range - dups.
    /// Opcode 0x80 - Duplicate 1st stack item
    DUP1 = 0x80,
    /// Opcode 0x81 - Duplicate 2nd stack item
    DUP2,
    /// Opcode 0x82 - Duplicate 3rd stack item
    DUP3,
    /// Opcode 0x83 - Duplicate 4th stack item
    DUP4,
    /// Opcode 0x84 - Duplicate 5th stack item
    DUP5,
    /// Opcode 0x85 - Duplicate 6th stack item
    DUP6,
    /// Opcode 0x86 - Duplicate 7th stack item
    DUP7,
    /// Opcode 0x87 - Duplicate 8th stack item
    DUP8,
    /// Opcode 0x88 - Duplicate 9th stack item
    DUP9,
    /// Opcode 0x89 - Duplicate 10th stack item
    DUP10,
    /// Opcode 0x8A - Duplicate 11th stack item
    DUP11,
    /// Opcode 0x8B - Duplicate 12th stack item
    DUP12,
    /// Opcode 0x8C - Duplicate 13th stack item
    DUP13,
    /// Opcode 0x8D - Duplicate 14th stack item
    DUP14,
    /// Opcode 0x8E - Duplicate 15th stack item
    DUP15,
    /// Opcode 0x8F - Duplicate 16th stack item
    DUP16,

    // 0x90 range - swaps.
    /// Opcode 0x90 - Exchange 1st and 2nd stack items
    SWAP1 = 0x90,
    /// Opcode 0x91 - Exchange 1st and 3rd stack items
    SWAP2,
    /// Opcode 0x92 - Exchange 1st and 4th stack items
    SWAP3,
    /// Opcode 0x93 - Exchange 1st and 5th stack items
    SWAP4,
    /// Opcode 0x94 - Exchange 1st and 6th stack items
    SWAP5,
    /// Opcode 0x95 - Exchange 1st and 7th stack items
    SWAP6,
    /// Opcode 0x96 - Exchange 1st and 8th stack items
    SWAP7,
    /// Opcode 0x97 - Exchange 1st and 9th stack items
    SWAP8,
    /// Opcode 0x98 - Exchange 1st and 10th stack items
    SWAP9,
    /// Opcode 0x99 - Exchange 1st and 11th stack items
    SWAP10,
    /// Opcode 0x9A - Exchange 1st and 12th stack items
    SWAP11,
    /// Opcode 0x9B - Exchange 1st and 13th stack items
    SWAP12,
    /// Opcode 0x9C - Exchange 1st and 14th stack items
    SWAP13,
    /// Opcode 0x9D - Exchange 1st and 15th stack items
    SWAP14,
    /// Opcode 0x9E - Exchange 1st and 16th stack items
    SWAP15,
    /// Opcode 0x9F - Exchange 1st and 17th stack items
    SWAP16,

    // 0xA0 range - logging ops.
    /// Opcode 0xA0 - Append log record with one topic
    LOG0 = 0xa0,
    /// Opcode 0xA1 - Append log record with two topics
    LOG1,
    /// Opcode 0xA2 - Append log record with three topics
    LOG2,
    /// Opcode 0xA3 - Append log record with four topics
    LOG3,
    /// Opcode 0xA4 - Append log record with five topics
    LOG4,

    // 0xA5 - 0xEF are invalid

    // 0xF0 range - closures.
    /// Opcode 0xF0 - Create a new account with associated code
    CREATE = 0xf0,
    /// Opcode 0xF1 - Message-call into an account
    CALL,
    /// Opcode 0xF2 - Message-call into this account with alternative account’s code
    CALLCODE,
    /// Opcode 0xF3 - Halt execution returning output data
    RETURN,
    /// Opcode 0xF4 - Message-call into this account with an alternative account’s code, but
    /// persisting the current values for sender and value
    DELEGATECALL,
    /// Opcode 0xF5 - Create a new account with associated code at a predictable address
    CREATE2,

    // 0xF6 - 0xF9 are invalid

    // 0xFA range - closures
    /// Opcode 0xFA - Static message-call into an account
    STATICCALL = 0xfa,

    // 0xFB - 0xFC are invalid

    // 0xfd range - closures
    /// Opcode 0xFD - Halt execution reverting state changes but returning data and remaining gas
    REVERT = 0xfd,
    /// Opcode 0xFE - Designated invalid instruction
    INVALID = 0xfe,
    /// Opcode 0xFF - Halt execution and register account for later deletion
    SELFDESTRUCT = 0xff,
}

// See comment in ./chain.rs
#[allow(clippy::derivable_impls)]
impl Default for Opcode {
    fn default() -> Self {
        Opcode::INVALID
    }
}

impl From<Opcode> for u8 {
    fn from(value: Opcode) -> Self {
        value as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::{value::StrDeserializer, IntoDeserializer};
    use std::collections::HashSet;

    // Taken from REVM:
    // https://github.com/bluealloy/revm/blob/f8ff6b330dce126ab9359ab8dde02ba1d09b9306/crates/interpreter/src/instructions/opcode.rs#L184
    const OPCODE_JUMPMAP: [Option<&'static str>; 256] = [
        /* 0x00 */ Some("STOP"),
        /* 0x01 */ Some("ADD"),
        /* 0x02 */ Some("MUL"),
        /* 0x03 */ Some("SUB"),
        /* 0x04 */ Some("DIV"),
        /* 0x05 */ Some("SDIV"),
        /* 0x06 */ Some("MOD"),
        /* 0x07 */ Some("SMOD"),
        /* 0x08 */ Some("ADDMOD"),
        /* 0x09 */ Some("MULMOD"),
        /* 0x0a */ Some("EXP"),
        /* 0x0b */ Some("SIGNEXTEND"),
        /* 0x0c */ None,
        /* 0x0d */ None,
        /* 0x0e */ None,
        /* 0x0f */ None,
        /* 0x10 */ Some("LT"),
        /* 0x11 */ Some("GT"),
        /* 0x12 */ Some("SLT"),
        /* 0x13 */ Some("SGT"),
        /* 0x14 */ Some("EQ"),
        /* 0x15 */ Some("ISZERO"),
        /* 0x16 */ Some("AND"),
        /* 0x17 */ Some("OR"),
        /* 0x18 */ Some("XOR"),
        /* 0x19 */ Some("NOT"),
        /* 0x1a */ Some("BYTE"),
        /* 0x1b */ Some("SHL"),
        /* 0x1c */ Some("SHR"),
        /* 0x1d */ Some("SAR"),
        /* 0x1e */ None,
        /* 0x1f */ None,
        /* 0x20 */ Some("SHA3"),
        /* 0x21 */ None,
        /* 0x22 */ None,
        /* 0x23 */ None,
        /* 0x24 */ None,
        /* 0x25 */ None,
        /* 0x26 */ None,
        /* 0x27 */ None,
        /* 0x28 */ None,
        /* 0x29 */ None,
        /* 0x2a */ None,
        /* 0x2b */ None,
        /* 0x2c */ None,
        /* 0x2d */ None,
        /* 0x2e */ None,
        /* 0x2f */ None,
        /* 0x30 */ Some("ADDRESS"),
        /* 0x31 */ Some("BALANCE"),
        /* 0x32 */ Some("ORIGIN"),
        /* 0x33 */ Some("CALLER"),
        /* 0x34 */ Some("CALLVALUE"),
        /* 0x35 */ Some("CALLDATALOAD"),
        /* 0x36 */ Some("CALLDATASIZE"),
        /* 0x37 */ Some("CALLDATACOPY"),
        /* 0x38 */ Some("CODESIZE"),
        /* 0x39 */ Some("CODECOPY"),
        /* 0x3a */ Some("GASPRICE"),
        /* 0x3b */ Some("EXTCODESIZE"),
        /* 0x3c */ Some("EXTCODECOPY"),
        /* 0x3d */ Some("RETURNDATASIZE"),
        /* 0x3e */ Some("RETURNDATACOPY"),
        /* 0x3f */ Some("EXTCODEHASH"),
        /* 0x40 */ Some("BLOCKHASH"),
        /* 0x41 */ Some("COINBASE"),
        /* 0x42 */ Some("TIMESTAMP"),
        /* 0x43 */ Some("NUMBER"),
        /* 0x44 */ Some("DIFFICULTY"),
        /* 0x45 */ Some("GASLIMIT"),
        /* 0x46 */ Some("CHAINID"),
        /* 0x47 */ Some("SELFBALANCE"),
        /* 0x48 */ Some("BASEFEE"),
        /* 0x49 */ None,
        /* 0x4a */ None,
        /* 0x4b */ None,
        /* 0x4c */ None,
        /* 0x4d */ None,
        /* 0x4e */ None,
        /* 0x4f */ None,
        /* 0x50 */ Some("POP"),
        /* 0x51 */ Some("MLOAD"),
        /* 0x52 */ Some("MSTORE"),
        /* 0x53 */ Some("MSTORE8"),
        /* 0x54 */ Some("SLOAD"),
        /* 0x55 */ Some("SSTORE"),
        /* 0x56 */ Some("JUMP"),
        /* 0x57 */ Some("JUMPI"),
        /* 0x58 */ Some("PC"),
        /* 0x59 */ Some("MSIZE"),
        /* 0x5a */ Some("GAS"),
        /* 0x5b */ Some("JUMPDEST"),
        /* 0x5c */ None,
        /* 0x5d */ None,
        /* 0x5e */ None,
        /* 0x5f */ Some("PUSH0"),
        /* 0x60 */ Some("PUSH1"),
        /* 0x61 */ Some("PUSH2"),
        /* 0x62 */ Some("PUSH3"),
        /* 0x63 */ Some("PUSH4"),
        /* 0x64 */ Some("PUSH5"),
        /* 0x65 */ Some("PUSH6"),
        /* 0x66 */ Some("PUSH7"),
        /* 0x67 */ Some("PUSH8"),
        /* 0x68 */ Some("PUSH9"),
        /* 0x69 */ Some("PUSH10"),
        /* 0x6a */ Some("PUSH11"),
        /* 0x6b */ Some("PUSH12"),
        /* 0x6c */ Some("PUSH13"),
        /* 0x6d */ Some("PUSH14"),
        /* 0x6e */ Some("PUSH15"),
        /* 0x6f */ Some("PUSH16"),
        /* 0x70 */ Some("PUSH17"),
        /* 0x71 */ Some("PUSH18"),
        /* 0x72 */ Some("PUSH19"),
        /* 0x73 */ Some("PUSH20"),
        /* 0x74 */ Some("PUSH21"),
        /* 0x75 */ Some("PUSH22"),
        /* 0x76 */ Some("PUSH23"),
        /* 0x77 */ Some("PUSH24"),
        /* 0x78 */ Some("PUSH25"),
        /* 0x79 */ Some("PUSH26"),
        /* 0x7a */ Some("PUSH27"),
        /* 0x7b */ Some("PUSH28"),
        /* 0x7c */ Some("PUSH29"),
        /* 0x7d */ Some("PUSH30"),
        /* 0x7e */ Some("PUSH31"),
        /* 0x7f */ Some("PUSH32"),
        /* 0x80 */ Some("DUP1"),
        /* 0x81 */ Some("DUP2"),
        /* 0x82 */ Some("DUP3"),
        /* 0x83 */ Some("DUP4"),
        /* 0x84 */ Some("DUP5"),
        /* 0x85 */ Some("DUP6"),
        /* 0x86 */ Some("DUP7"),
        /* 0x87 */ Some("DUP8"),
        /* 0x88 */ Some("DUP9"),
        /* 0x89 */ Some("DUP10"),
        /* 0x8a */ Some("DUP11"),
        /* 0x8b */ Some("DUP12"),
        /* 0x8c */ Some("DUP13"),
        /* 0x8d */ Some("DUP14"),
        /* 0x8e */ Some("DUP15"),
        /* 0x8f */ Some("DUP16"),
        /* 0x90 */ Some("SWAP1"),
        /* 0x91 */ Some("SWAP2"),
        /* 0x92 */ Some("SWAP3"),
        /* 0x93 */ Some("SWAP4"),
        /* 0x94 */ Some("SWAP5"),
        /* 0x95 */ Some("SWAP6"),
        /* 0x96 */ Some("SWAP7"),
        /* 0x97 */ Some("SWAP8"),
        /* 0x98 */ Some("SWAP9"),
        /* 0x99 */ Some("SWAP10"),
        /* 0x9a */ Some("SWAP11"),
        /* 0x9b */ Some("SWAP12"),
        /* 0x9c */ Some("SWAP13"),
        /* 0x9d */ Some("SWAP14"),
        /* 0x9e */ Some("SWAP15"),
        /* 0x9f */ Some("SWAP16"),
        /* 0xa0 */ Some("LOG0"),
        /* 0xa1 */ Some("LOG1"),
        /* 0xa2 */ Some("LOG2"),
        /* 0xa3 */ Some("LOG3"),
        /* 0xa4 */ Some("LOG4"),
        /* 0xa5 */ None,
        /* 0xa6 */ None,
        /* 0xa7 */ None,
        /* 0xa8 */ None,
        /* 0xa9 */ None,
        /* 0xaa */ None,
        /* 0xab */ None,
        /* 0xac */ None,
        /* 0xad */ None,
        /* 0xae */ None,
        /* 0xaf */ None,
        /* 0xb0 */ None,
        /* 0xb1 */ None,
        /* 0xb2 */ None,
        /* 0xb3 */ None,
        /* 0xb4 */ None,
        /* 0xb5 */ None,
        /* 0xb6 */ None,
        /* 0xb7 */ None,
        /* 0xb8 */ None,
        /* 0xb9 */ None,
        /* 0xba */ None,
        /* 0xbb */ None,
        /* 0xbc */ None,
        /* 0xbd */ None,
        /* 0xbe */ None,
        /* 0xbf */ None,
        /* 0xc0 */ None,
        /* 0xc1 */ None,
        /* 0xc2 */ None,
        /* 0xc3 */ None,
        /* 0xc4 */ None,
        /* 0xc5 */ None,
        /* 0xc6 */ None,
        /* 0xc7 */ None,
        /* 0xc8 */ None,
        /* 0xc9 */ None,
        /* 0xca */ None,
        /* 0xcb */ None,
        /* 0xcc */ None,
        /* 0xcd */ None,
        /* 0xce */ None,
        /* 0xcf */ None,
        /* 0xd0 */ None,
        /* 0xd1 */ None,
        /* 0xd2 */ None,
        /* 0xd3 */ None,
        /* 0xd4 */ None,
        /* 0xd5 */ None,
        /* 0xd6 */ None,
        /* 0xd7 */ None,
        /* 0xd8 */ None,
        /* 0xd9 */ None,
        /* 0xda */ None,
        /* 0xdb */ None,
        /* 0xdc */ None,
        /* 0xdd */ None,
        /* 0xde */ None,
        /* 0xdf */ None,
        /* 0xe0 */ None,
        /* 0xe1 */ None,
        /* 0xe2 */ None,
        /* 0xe3 */ None,
        /* 0xe4 */ None,
        /* 0xe5 */ None,
        /* 0xe6 */ None,
        /* 0xe7 */ None,
        /* 0xe8 */ None,
        /* 0xe9 */ None,
        /* 0xea */ None,
        /* 0xeb */ None,
        /* 0xec */ None,
        /* 0xed */ None,
        /* 0xee */ None,
        /* 0xef */ None,
        /* 0xf0 */ Some("CREATE"),
        /* 0xf1 */ Some("CALL"),
        /* 0xf2 */ Some("CALLCODE"),
        /* 0xf3 */ Some("RETURN"),
        /* 0xf4 */ Some("DELEGATECALL"),
        /* 0xf5 */ Some("CREATE2"),
        /* 0xf6 */ None,
        /* 0xf7 */ None,
        /* 0xf8 */ None,
        /* 0xf9 */ None,
        /* 0xfa */ Some("STATICCALL"),
        /* 0xfb */ None,
        /* 0xfc */ None,
        /* 0xfd */ Some("REVERT"),
        /* 0xfe */ Some("INVALID"),
        /* 0xff */ Some("SELFDESTRUCT"),
    ];

    #[test]
    fn all() {
        let len = Opcode::COUNT;
        let mut found = HashSet::with_capacity(len);

        for (i, mnemonic) in OPCODE_JUMPMAP.iter().enumerate() {
            let Some(mnemonic) = *mnemonic else { continue };
            let parsed = mnemonic.parse::<Opcode>().unwrap();
            if !found.insert(parsed) {
                panic!("Duplicate Opcode: {mnemonic:?} => {parsed}")
            }

            assert_eq!(i, parsed as usize);
            assert_eq!(Opcode::try_from(i as u8).unwrap(), parsed);
            assert_eq!(OPCODE_JUMPMAP[i].unwrap(), mnemonic);

            // strum
            assert_eq!(parsed.as_ref(), mnemonic);
            assert_eq!(parsed.to_string(), mnemonic);

            // serde
            let de: StrDeserializer<'_, serde::de::value::Error> = mnemonic.into_deserializer();
            let serde = Opcode::deserialize(de).unwrap();
            assert_eq!(serde, parsed);
            assert_eq!(serde_json::to_string(&serde).unwrap(), format!("\"{mnemonic}\""));
        }

        assert_eq!(found.len(), len);
    }
}
