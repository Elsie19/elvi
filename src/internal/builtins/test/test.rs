//! Main caller.

use super::execute::execute;
use crate::internal::status::ReturnCode;
use crate::internal::tree::TestOptions;
use crate::internal::variables::Variables;

/// The internal code that runs when the `test` builtin is run.
#[must_use]
pub fn main(invert: bool, to_do: TestOptions, variables: &Variables) -> ReturnCode {
    execute(invert, to_do, variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::variables::ElviType;

    #[test]
    fn test_file() {
        let variables = Variables::default();
        assert_eq!(
            builtin_test(
                false,
                TestOptions::RegularFileExists(ElviType::String("/etc/passwd".into())),
                &variables
            ),
            true.into()
        )
    }

    #[test]
    fn writable_test() {
        let variables = Variables::default();
        assert_eq!(
            builtin_test(
                false,
                TestOptions::FileExistsWritable(ElviType::String("/etc/passwd".into())),
                &variables
            ),
            false.into()
        )
    }

    #[test]
    fn test_strings_equals() {
        let variables = Variables::default();
        assert_eq!(
            builtin_test(
                false,
                TestOptions::String1IsString2((
                    ElviType::String("foo".into()),
                    ElviType::String("foo".into()),
                )),
                &variables,
            ),
            true.into()
        )
    }

    #[test]
    fn directory_exists() {
        let variables = Variables::default();
        assert_eq!(
            builtin_test(
                false,
                TestOptions::DirectoryExists(ElviType::String("/etc/".into())),
                &variables
            ),
            true.into()
        )
    }

    #[test]
    fn directory_not_exists() {
        let variables = Variables::default();
        assert_eq!(
            builtin_test(
                true,
                TestOptions::DirectoryExists(ElviType::String("/not_exists/".into())),
                &variables
            ),
            true.into()
        )
    }
}
