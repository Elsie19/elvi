//! Contains modules relating to interfacing between Elvi and its internals.
/// Contains modules relating to handling commands.
pub mod commands;
/// Contains modules relating to builtins.
///
/// Here are the instructions that must be done to implement a builtin:
///
/// 1. Create a grammar file for it in `src/parse/internals/builtins.pest`.
/// 2. Add it to `src/parse/internals/base.pest`.
/// 3. Add an accompanying handler in [`super::parse::grammar::ElviParser`].
/// 4. Take that return type without the `Ok()` and create a builtin variant in [`tree::Builtins`].
/// 5. Create a new module in `src/internal.rs` inside the `builtins` module.
/// 6. Create the corresponding file in `src/internal/builtins/`.
/// 7. Create a function called `builtin_{}` and start working.
/// 8. Go back to `src/parse/grammar.rs` and find [`tree::Builtins`] and add your function there in
///    the match statement. Remember, the `builtin_{}` is only for how the program interacts with
///    its given data, in `grammar.rs` you call the function and handle the environment from there.
pub mod builtins {
    /// The `dbg` builtin.
    pub mod dbg;
    /// The `exit` builtin.
    pub mod exit;
    /// The `unset` builtin.
    pub mod unset;
}
/// Contains modules relating to handling error codes.
pub mod status;
/// Contains modules relating to executing Elvi code.
pub mod tree;
/// Contains modules relating to handling variables.
pub mod variables;
