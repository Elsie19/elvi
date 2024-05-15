//! Contains modules relating to interfacing between Elvi and its internals.
/// Contains modules relating to handling commands.
pub mod commands;
/// Contains modules relating to builtins.
pub mod builtins {
    /// The `dbg` builtin.
    pub mod dbg;
    /// The `exit` builtin.
    pub mod exit;
}
/// Contains modules relating to handling error codes.
pub mod status;
/// Contains modules relating to executing Elvi code.
pub mod tree;
/// Contains modules relating to handling variables.
pub mod variables;
