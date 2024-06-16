/// Struct to handle the global environment.
pub struct Env {
    in_function: bool,
    pub subshells_in: u32,
}

impl Env {
    /// Set whether inside function or not.
    pub fn set_function(&mut self, in_function: bool) -> bool {
        self.in_function = in_function;
        self.in_function
    }

    /// Query whether inside a function or not.
    pub fn in_function(&self) -> bool {
        self.in_function
    }
}

impl Default for Env {
    fn default() -> Self {
        Self {
            in_function: false,
            subshells_in: 1,
        }
    }
}
