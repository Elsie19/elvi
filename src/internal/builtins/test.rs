use std::fs;
use std::path::Path;

use crate::internal::status::ReturnCode;
use crate::internal::tree::TestOptions;
use crate::internal::variables::Variables;

/// The internal code that runs when the `test` builtin is run.
pub fn builtin_test(to_do: TestOptions, variables: &Variables) -> ReturnCode {
    match to_do {
        TestOptions::String1IsString2((s1, s2)) => (s1.eval_escapes().eval_variables(variables)
            == s2.eval_escapes().eval_variables(variables))
        .into(),
        TestOptions::String1IsNotString2((s1, s2)) => (s1.eval_escapes().eval_variables(variables)
            != s2.eval_escapes().eval_variables(variables))
        .into(),
        TestOptions::String1BeforeString2ASCII((s1, s2)) => {
            (s1.eval_escapes().eval_variables(variables).to_string()
                > s2.eval_escapes().eval_variables(variables).to_string())
            .into()
        }
        TestOptions::String1AfterString2ASCII((s1, s2)) => {
            (s1.eval_escapes().eval_variables(variables).to_string()
                < s2.eval_escapes().eval_variables(variables).to_string())
            .into()
        }
        TestOptions::Int1EqualsInt2Algebraically((n1, n2)) => (n1
            .eval_escapes()
            .eval_variables(variables)
            .to_string()
            .parse::<usize>()
            .unwrap()
            == n2
                .eval_escapes()
                .eval_variables(variables)
                .to_string()
                .parse::<usize>()
                .unwrap())
        .into(),
        TestOptions::Int1LessThanInt2Algebraically((n1, n2)) => (n1
            .eval_escapes()
            .eval_variables(variables)
            .to_string()
            .parse::<usize>()
            .unwrap()
            < n2.eval_escapes()
                .eval_variables(variables)
                .to_string()
                .parse::<usize>()
                .unwrap())
        .into(),
        TestOptions::Int1NotEqualsInt2Algebraically((n1, n2)) => (n1
            .eval_escapes()
            .eval_variables(variables)
            .to_string()
            .parse::<usize>()
            .unwrap()
            != n2
                .eval_escapes()
                .eval_variables(variables)
                .to_string()
                .parse::<usize>()
                .unwrap())
        .into(),
        TestOptions::Int1LessEqualInt2Algebraically((n1, n2)) => (n1
            .eval_escapes()
            .eval_variables(variables)
            .to_string()
            .parse::<usize>()
            .unwrap()
            <= n2
                .eval_escapes()
                .eval_variables(variables)
                .to_string()
                .parse::<usize>()
                .unwrap())
        .into(),
        TestOptions::Int1GreaterThanInt2Algebraically((n1, n2)) => (n1
            .eval_escapes()
            .eval_variables(variables)
            .to_string()
            .parse::<usize>()
            .unwrap()
            > n2.eval_escapes()
                .eval_variables(variables)
                .to_string()
                .parse::<usize>()
                .unwrap())
        .into(),
        TestOptions::Int1GreaterEqualInt2Algebraically((n1, n2)) => (n1
            .eval_escapes()
            .eval_variables(variables)
            .to_string()
            .parse::<usize>()
            .unwrap()
            >= n2
                .eval_escapes()
                .eval_variables(variables)
                .to_string()
                .parse::<usize>()
                .unwrap())
        .into(),
        TestOptions::RegularFileExists(file) => match fs::metadata(Path::new(
            &file.eval_escapes().eval_variables(variables).to_string(),
        )) {
            Ok(metadata) => metadata.is_file().into(),
            Err(_) => false,
        }
        .into(),
        TestOptions::AnyFileExists(file) => {
            (Path::new(&file.eval_escapes().eval_variables(variables).to_string()))
                .exists()
                .into()
        }
        _ => todo!(),
    }
}
