use std::fs;
use std::os::unix::fs::PermissionsExt;
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
            Ok(metadata) => metadata.is_file(),
            Err(_) => false,
        }
        .into(),
        TestOptions::AnyFileExists(file) => {
            (Path::new(&file.eval_escapes().eval_variables(variables).to_string()))
                .exists()
                .into()
        }
        TestOptions::DirectoryExists(dir) => {
            fs::metadata(dir.eval_escapes().eval_variables(variables).to_string())
                .unwrap()
                .is_dir()
                .into()
        }
        TestOptions::BlockFileExists(_file)
        | TestOptions::CharacterFileExists(_file)
        | TestOptions::GroupIDFlagSetExists(_file) => todo!(),
        TestOptions::SymbolicLinkExists(link) => {
            match fs::symlink_metadata(link.eval_escapes().eval_variables(variables).to_string()) {
                Ok(metadata) => metadata.file_type().is_symlink().into(),
                Err(_) => false.into(),
            }
        }
        TestOptions::StickyBitSetExists(file) => {
            match fs::metadata(file.eval_escapes().eval_variables(variables).to_string()) {
                Ok(metadata) => {
                    let permissions = metadata.permissions();
                    (permissions.mode() & 0o1000 != 0).into()
                }
                Err(_) => false.into(),
            }
        }
        TestOptions::StringZero(stringo) => stringo
            .eval_escapes()
            .eval_variables(variables)
            .to_string()
            .is_empty()
            .into(),
        TestOptions::StringNonZero(stringo) => !<bool as Into<ReturnCode>>::into(
            stringo
                .eval_escapes()
                .eval_variables(variables)
                .to_string()
                .is_empty(),
        ),
        _ => todo!(),
    }
}
