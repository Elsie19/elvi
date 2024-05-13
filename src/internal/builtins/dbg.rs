use crate::internal::variables::Variable;

pub fn builtin_dbg(name: String, var: Variable) {
    println!("Name is: '{}'\nVar contents are: {:?}", name, var);
}
