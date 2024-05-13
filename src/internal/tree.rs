use super::variables::Variable;

#[derive(Debug)]
pub struct Tree {
    pub exprs: Vec<Actions>,
}

#[derive(Debug)]
pub enum Actions {
    ChangeVariable((String, Variable)),
}

impl Tree {
    pub fn add_action(&mut self, action: Actions) {
        self.exprs.push(action)
    }

    pub fn new() -> Self {
        Self { exprs: vec![] }
    }
}
