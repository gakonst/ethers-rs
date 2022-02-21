//! algorithms used to detect and divide a graph into components.
//!
//! In graph theory, a _component_ of a graph is a connected subgraph that is not part of a larger
//! connected subgraph.
//!
//! In a [`Project`]'s dependency graph every file is represented as a `Node`. The edges in the
//! graph represent the _import_ relationship between the nodes: `A.sol -> B.sol` if `A.sol` imports
//! `B.sol`.
//! In general, dependency graphs are directed acyclic graphs, solidity even allows cyclical
//! imports: `A.sol <-> B.sol`.
//!
//! A graph is said to be connected if every pair of nodes in the graph is connected. This means
//! that there is a path between every pair of nodes. In other words, if all files of the project
//! are related to one another then the dependency graph of the project is connected.
//!
//! ## Examples
//!
//! The dependency graph of a project
//!   - with a single file with _no_ imports is connected.
//!   - with files `A.sol`, `B.sol` and `C.sol` is connected if
//!     - `A` imports `B` and
//!     - `B` imports `C`
//!    so that the graph is `A -> B -> C`
//!
//! For compiling a project, this means that a project with a connected dependency graph cannot be
//! compiled in parallel without overhead. Because the graph contains exactly one component, the
//! whole dependency graph itself.
//! However, if all files are not connected, then there are at least 2 components that are
//! completely unconnected.  If for example `A` only imports `B` and `B` does _not_ import `C` then
//! the dependency graph contains two components: [`A -> B`, `C`] which can be compiled in parallel
//! without overhead as there is no relationship between the components.
//!
//! The computational problem of finding a small set of edges to add or remove from a graph to
//! transform it into a cluster graph is called cluster editing.
//!
//! This module contains algorithms to detect all components of a project's dependency graph and to
//! transform connected graphs into a cluster graph based on certain heuristics
