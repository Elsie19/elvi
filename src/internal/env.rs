use super::commands::HowRun;

/// Struct to handle the global environment.
pub struct Env {
    in_function: bool,
    pub subshells_in: u32,
    pub stdout: String,
    pub stderr: String,
}

pub enum Std {
    Out,
    Err,
}

impl Env {
    /// Set whether inside function or not.
    pub fn set_function(&mut self, in_function: bool) {
        self.in_function = in_function;
    }

    /// Query whether inside a function or not.
    #[must_use]
    pub fn in_function(&self) -> bool {
        self.in_function
    }

    /// Append or print text to screen
    pub fn print(&mut self, whereto: Std, how: HowRun, text: &str) {
        match how {
            HowRun::RealTime => match whereto {
                Std::Out => print!("{text}"),
                Std::Err => eprint!("{text}"),
            },
            HowRun::Piped | HowRun::Substitution => match whereto {
                Std::Out => self.stdout.push_str(text),
                Std::Err => self.stderr.push_str(text),
            },
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self {
            in_function: false,
            subshells_in: 1,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}
