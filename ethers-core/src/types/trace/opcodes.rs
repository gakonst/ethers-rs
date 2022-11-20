use serde::{Deserialize, Serialize};

// opcode descriptions taken from evm.codes https://github.com/comitylabs/evm.codes/blob/bc7f102808055d88365559d40c190c5bd6d164c3/opcodes.json
// https://github.com/ethereum/go-ethereum/blob/2b1299b1c006077c56ecbad32e79fc16febe3dd6/core/vm/opcodes.go
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
/// Name of executed EVM opcode
pub enum OpCode {
    // 0x0 range - arithmetic ops.
    /// Opcode 0x0 - Halts execution
    STOP,
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
    LT,
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
    SHA3,

    // 0x21 - 0x2F are invalid

    // 0x30 range - closure state.
    /// Opcode 0x30 - Get address of currently executing account
    ADDRESS,
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
    BLOCKHASH,
    /// Opcode 0x41 - Get the block’s beneficiary address
    COINBASE,
    /// Opcode 0x42 - Get the block’s timestamp
    TIMESTAMP,
    /// Opcode 0x43 - Get the block’s number
    NUMBER,
    /// Opcode 0x44 - Get the block’s difficulty
    DIFFICULTY,
    //RANDOM,      // 0x44 // Same as DIFFICULTY
    //PREVRANDAO,  // 0x44 // Same as DIFFICULTY
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
    POP,
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

    // 0x5C - 0x5F are invalid

    // 0x60 range - pushes.
    // PUSH0,    // 0x5F (https://eips.ethereum.org/EIPS/eip-3855)
    /// Opcode 0x60 - Place 1 byte item on stack
    PUSH1,
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
    DUP1,
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
    /// Opcode 0x90 - Exchange 1st and 1st stack items
    SWAP1,
    /// Opcode 0x91 - Exchange 1st and 2nd stack items
    SWAP2,
    /// Opcode 0x92 - Exchange 1st and 3rd stack items
    SWAP3,
    /// Opcode 0x93 - Exchange 1st and 4th stack items
    SWAP4,
    /// Opcode 0x94 - Exchange 1st and 5th stack items
    SWAP5,
    /// Opcode 0x95 - Exchange 1st and 6th stack items
    SWAP6,
    /// Opcode 0x96 - Exchange 1st and 7th stack items
    SWAP7,
    /// Opcode 0x97 - Exchange 1st and 8th stack items
    SWAP8,
    /// Opcode 0x98 - Exchange 1st and 9th stack items
    SWAP9,
    /// Opcode 0x99 - Exchange 1st and 10th stack items
    SWAP10,
    /// Opcode 0x9A - Exchange 1st and 11th stack items
    SWAP11,
    /// Opcode 0x9B - Exchange 1st and 12th stack items
    SWAP12,
    /// Opcode 0x9C - Exchange 1st and 13th stack items
    SWAP13,
    /// Opcode 0x9D - Exchange 1st and 14th stack items
    SWAP14,
    /// Opcode 0x9E - Exchange 1st and 15th stack items
    SWAP15,
    /// Opcode 0x9F - Exchange 1st and 16th stack items
    SWAP16,

    // 0xA0 range - logging ops.
    /// Opcode 0xA0 - Append log record with one topic
    LOG0,
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
    CREATE,
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
    STATICCALL,

    // 0xFB - 0xFC are invalid

    // 0xfd range - closures
    /// Opcode 0xFD - Halt execution reverting state changes but returning data and remaining gas
    REVERT,
    #[default]
    /// Opcode 0xFE - Designated invalid instruction
    INVALID,
    /// Opcode 0xFF - Halt execution and register account for later deletion
    SELFDESTRUCT,
}
