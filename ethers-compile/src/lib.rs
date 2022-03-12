/// Defines the Compiler Interface
pub mod compiler;

/// Implements Compiler for Solc
pub mod solc;

// Reexport everything from the project module
mod project;
pub use project::*;