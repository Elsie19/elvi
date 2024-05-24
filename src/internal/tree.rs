use std::io::Write;
use std::{io, process::Command};

use crate::internal::commands::CommandError;
use crate::internal::status::ReturnCode;

use super::{
    commands::{Commands, ExternalCommand},
    variables::{ElviGlobal, ElviType, Variable, Variables},
};

#[derive(Debug, Clone)]
/// A list of possible actions a line can cause.
pub enum Actions {
    /// Change/create a variable.
    ChangeVariable((String, Variable)),
    /// Execute a builtin.
    Builtin(Builtins),
    /// Run a command.
    Command(Vec<String>),
    /// If statement
    IfStatement(Box<Conditional>),
    /// For loop
    ForLoop(Loop),
    /// Do nothing.
    Null,
}

#[derive(Debug, Clone)]
/// A list of builtins and their parameters.
pub enum Builtins {
    /// Just needs a variable name.
    Dbg(String),
    /// Just needs a variable name.
    Unset(String),
    /// Will exit with `0` if not given data, and if so, attempt to parse into a number.
    Exit(Option<ElviType>),
    /// Will display commands if no argument given, will regenerate if `-r` is passed.
    Hash(Option<ElviType>),
    /// Needs a path, empty, or dash.
    Cd(Option<ElviType>),
    /// Needs something to test and a bool (! for example).
    Test(bool, TestOptions),
    /// Can take nothing or a list of stuff
    Echo(Option<Vec<ElviType>>),
}

#[derive(Debug, Clone)]
/// A struct for conditional execution.
pub struct Conditional {
    /// The primary condition to execute.
    pub condition: Actions,
    /// The resulting code that is executed if [`Conditional::condition`] succeeds.
    pub then_block: Vec<Actions>,
    /// Optional else block if [`Conditional::condition`] fails.
    pub else_block: Option<Vec<Actions>>,
}

#[derive(Debug, Clone)]
/// A struct for loop execution.
pub struct Loop {
    /// Variable to update
    pub variable: ElviType,
    /// Vector of elements to loop over.
    pub elements: Vec<ElviType>,
    /// The resulting code that is executed every [`Loop::elements`].
    pub do_block: Vec<Actions>,
}

#[derive(Debug, Clone)]
/// A list of things `test` can do.
pub enum TestOptions {
    /// `-b file`
    BlockFileExists(ElviType),
    /// `-c file`
    CharacterFileExists(ElviType),
    /// `-d file`
    DirectoryExists(ElviType),
    /// `-e file`
    AnyFileExists(ElviType),
    /// `-f file`
    RegularFileExists(ElviType),
    /// `-g file`
    GroupIDFlagSetExists(ElviType),
    /// `-h file` & `-L file`
    SymbolicLinkExists(ElviType),
    /// `-k file`
    StickyBitSetExists(ElviType),
    /// `-n file`
    StringNonZero(ElviType),
    /// `-p file`
    NamedPipeExists(ElviType),
    /// `-r file`
    ReadableFileExists(ElviType),
    /// `-s file`
    FileExistsGreaterThanZero(ElviType),
    /// `-t file_descriptor`
    FDDescriptorNumberOpened(ElviType),
    /// `-u file`
    FileExistsUserIDSet(ElviType),
    /// `-w file`
    FileExistsWritable(ElviType),
    /// `-x file`
    FileExistsExecutable(ElviType),
    /// `-z string`
    StringZero(ElviType),
    /// `-O file`
    FileExistsOwnerEffectiveUserID(ElviType),
    /// `-G file`
    FileExistsOwnerEffectiveGroupID(ElviType),
    /// `-S file`
    FileExistsSocket(ElviType),
    /// `file1 -nt file2`
    File1NewerThanFile2((ElviType, ElviType)),
    /// `file1 -ot file2`
    File1OlderThanFile2((ElviType, ElviType)),
    /// `file1 -ef file2`
    File1SameAsFile2((ElviType, ElviType)),
    /// `string`
    StringNotNull(ElviType),
    /// `s1 = s2`
    String1IsString2((ElviType, ElviType)),
    /// `s1 != s2`
    String1IsNotString2((ElviType, ElviType)),
    /// `s1 < s2`
    String1BeforeString2ASCII((ElviType, ElviType)),
    /// `s1 > s2`
    String1AfterString2ASCII((ElviType, ElviType)),
    /// `n1 -eq n2`
    Int1EqualsInt2Algebraically((ElviType, ElviType)),
    /// `n1 -ne n2`
    Int1NotEqualsInt2Algebraically((ElviType, ElviType)),
    /// `n1 -gt n2`
    Int1GreaterThanInt2Algebraically((ElviType, ElviType)),
    /// `n1 -lt n2`
    Int1LessThanInt2Algebraically((ElviType, ElviType)),
    /// `n1 -ge n2`
    Int1GreaterEqualInt2Algebraically((ElviType, ElviType)),
    /// `n1 -le n2`
    Int1LessEqualInt2Algebraically((ElviType, ElviType)),
}

/// Function to change/assign a variable.
///
/// I added this because putting everything into `grammar.rs` was too much work and tedious.
pub fn change_variable(
    variables: &mut Variables,
    commands: &Commands,
    lvl: u32,
    name: String,
    var: &Variable,
) {
    // Makes shit easier to deal with.
    let mut var = var.clone();
    match var.get_value() {
        goopy @ ElviType::VariableSubstitution(_) => {
            // Goopy will save us!!!
            var.change_contents(goopy.eval_variables(variables));
            change_variable(variables, commands, lvl, name, &var);
        }
        ElviType::String(_we_dont_care) => {
            // First let's get the level because while parsing we assume a certain variable level that is
            // not true to the actual evaluating.
            if var.get_lvl() != ElviGlobal::Global {
                var.change_lvl(lvl);
            }
            match variables.set_variable(name, var) {
                Ok(()) => {}
                // Now in
                // <https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_08_01>,
                // it says that a variable assignment error on the interactive shell can continue,
                // but during a non-interactive shell, it must exact, and since Elvi is solely a
                // non-interactive shell, that's what we'll do.
                Err(oops) => {
                    eprintln!("{oops}");
                    std::process::exit(ReturnCode::MISUSE.into());
                }
            }
        }
        ElviType::CommandSubstitution(x) => {
            //TODO: Interpolate the variables if any
            let cmd_to_run = ExternalCommand::string_to_command(x);
            // Set variable to empty if we can't get the command
            if commands.get_path(&cmd_to_run.cmd).is_none() {
                eprintln!(
                    "{}",
                    CommandError::NotFound {
                        name: cmd_to_run.cmd,
                    }
                );
                if var.get_lvl() != ElviGlobal::Global {
                    var.change_lvl(lvl);
                }
                match variables.set_variable(
                    name,
                    Variable::oneshot_var(
                        &ElviType::String(String::new()),
                        var.get_modification_status(),
                        var.get_lvl(),
                        var.get_line(),
                    ),
                ) {
                    Ok(()) => {}
                    Err(oops) => eprintln!("{oops}"),
                }
            } else {
                // Let's get our environmental variables ready
                let filtered_env = variables.get_environmentals();
                // Full path, this is important as we don't want Command::new()
                // to be fooling around with it's own PATH, even though we
                // override it.
                let patho = commands.get_path(&cmd_to_run.cmd).unwrap();
                // This is a doozy but what we're doing is creating a command that takes the full
                // path to our program, clears it's own environment, inserts ours, set's the
                // current directory to ours as well.
                let cmd = Command::new(patho.to_str().unwrap())
                    .args(if cmd_to_run.args.is_none() {
                        vec![]
                    } else {
                        cmd_to_run.args.unwrap()
                    })
                    .env_clear()
                    .current_dir(
                        variables
                            .get_variable("PWD")
                            .unwrap()
                            .get_value()
                            .to_string(),
                    )
                    .envs(filtered_env)
                    .output()
                    .expect("oops");

                if !cmd.stderr.is_empty() {
                    io::stderr().write_all(&cmd.stderr).unwrap();
                }
                let mut var = var.clone();
                match variables.set_variable(
                    name,
                    var.change_contents(ElviType::String(
                        // POSIX says (somewhere trust me) that stderr shouldn't be in the variable
                        // assignment if it comes up.
                        std::str::from_utf8(&cmd.stdout)
                            .unwrap()
                            // This is to conform to
                            // <https://www.gnu.org/software/bash/manual/html_node/Command-Substitution.html>,
                            // specifically the part about trailing newlines deleted. This is from
                            // the bash manual but it is the same in POSIX, I've checked.
                            .trim_end_matches('\n')
                            .to_string(),
                    ))
                    .clone(),
                ) {
                    Ok(()) => {}
                    Err(oops) => eprintln!("{oops}"),
                }
            }
        }
        _ => unimplemented!("Give me a break please"),
    }
}
