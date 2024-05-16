//! Contains modules relating to interfacing between Elvi and its internals.
/// Contains modules relating to handling commands.
pub mod commands;
/// Contains modules relating to builtins.
///
/// # Creating a builtin
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
///
/// # Notes
/// All builtins defined by the [POSIX specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html) should function identically, and other builtins not related to these can do whatever.
pub mod builtins {
    /// The `dbg` builtin (not POSIX).
    ///
    /// # Notes
    /// This builtin is not in the POSIX spec and is an addition to Elvi. It should function mostly
    /// like `declare -p` from Bash, but not always.
    pub mod dbg;
    /// The `exit` builtin.
    ///
    /// Conforming to
    /// <https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#exit>
    pub mod exit;
    /// The `unset` builtin.
    ///
    /// Conforming to
    /// <https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#unset>
    ///
    /// # Notes
    /// Currently does not implement the following flags/behaviors:
    /// 1. `-v` flag for variables.
    /// 2. `-f` flag for functions.
    pub mod unset;
}
/// Contains modules relating to handling error codes.
pub mod status;
/// Contains modules relating to executing Elvi code.
pub mod tree;
/// Contains modules relating to handling variables.
pub mod variables;
