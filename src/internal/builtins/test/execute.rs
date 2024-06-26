//! Main logic.

use crate::internal::{status::ReturnCode, tree::TestOptions, variables::Variables};

use libc::isatty;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Execute the test options.
pub fn execute(invert: bool, to_do: TestOptions, variables: &Variables) -> ReturnCode {
    let ret = match to_do {
        TestOptions::String1IsString2((s1, s2)) => (s1.eval_escapes().eval_variables(variables)
            == s2.eval_escapes().eval_variables(variables))
        .into(),
        TestOptions::String1IsNotString2((s1, s2)) => {
            !execute(invert, TestOptions::String1IsString2((s1, s2)), variables)
        }
        TestOptions::String1BeforeString2ASCII((s1, s2)) => {
            (s1.eval_escapes().eval_variables(variables).to_string()
                > s2.eval_escapes().eval_variables(variables).to_string())
            .into()
        }
        TestOptions::String1AfterString2ASCII((s1, s2)) => !execute(
            invert,
            TestOptions::String1BeforeString2ASCII((s1, s2)),
            variables,
        ),
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
        TestOptions::Int1NotEqualsInt2Algebraically((n1, n2)) => !execute(
            invert,
            TestOptions::Int1EqualsInt2Algebraically((n1, n2)),
            variables,
        ),
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
        TestOptions::Int1GreaterThanInt2Algebraically((n1, n2)) => !execute(
            invert,
            TestOptions::Int1LessThanInt2Algebraically((n1, n2)),
            variables,
        ),
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
        TestOptions::RegularFileExists(file) => {
            match fs::metadata(file.eval_escapes().eval_variables(variables).to_string()) {
                Ok(metadata) => metadata.is_file(),
                Err(_) => false,
            }
            .into()
        }
        TestOptions::AnyFileExists(file) => {
            (Path::new(&file.eval_escapes().eval_variables(variables).to_string()))
                .exists()
                .into()
        }
        TestOptions::DirectoryExists(dir) => {
            match fs::metadata(dir.eval_escapes().eval_variables(variables).to_string()) {
                Ok(metadata) => metadata.is_dir(),
                Err(_) => false,
            }
            .into()
        }
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
        TestOptions::StringNonZero(stringo) | TestOptions::StringNotNull(stringo) => {
            !execute(invert, TestOptions::StringZero(stringo), variables)
        }
        TestOptions::ReadableFileExists(file) => {
            match File::open(file.eval_escapes().eval_variables(variables).to_string()) {
                Ok(mut file_p) => {
                    // Let's read *1* byte
                    let mut buffer = [0; 1];
                    match file_p.read_exact(&mut buffer) {
                        Ok(()) => true.into(),
                        Err(_) => false.into(),
                    }
                }
                Err(_) => false.into(),
            }
        }
        TestOptions::FileExistsGreaterThanZero(file) => {
            match fs::metadata(file.eval_escapes().eval_variables(variables).to_string()) {
                Ok(handle) => (handle.len() > 0).into(),
                Err(_) => false.into(),
            }
        }
        TestOptions::NamedPipeExists(file) => {
            if let Ok(metadata) =
                fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                metadata.file_type().is_fifo().into()
            } else {
                false.into()
            }
        }
        TestOptions::FileExistsWritable(file) => {
            if let Ok(metadata) =
                fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                (metadata.permissions().readonly()).into()
            } else {
                false.into()
            }
        }
        TestOptions::FileExistsExecutable(file) => {
            if let Ok(metadata) =
                fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                (metadata.permissions().mode() & 0o111 != 0).into()
            } else {
                false.into()
            }
        }
        TestOptions::BlockFileExists(file) => {
            if let Ok(metadata) =
                fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                (metadata.file_type().is_block_device()).into()
            } else {
                false.into()
            }
        }
        TestOptions::CharacterFileExists(file) => {
            if let Ok(metadata) =
                fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                (metadata.file_type().is_char_device()).into()
            } else {
                false.into()
            }
        }
        TestOptions::GroupIDFlagSetExists(file) => {
            if let Ok(metadata) =
                fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                (metadata.permissions().mode() & 0x2000 != 0).into()
            } else {
                false.into()
            }
        }
        TestOptions::FileExistsUserIDSet(file) => {
            if let Ok(metadata) =
                fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                (metadata.permissions().mode() & 0x4000 != 0).into()
            } else {
                false.into()
            }
        }
        TestOptions::FDDescriptorNumberOpened(number) => unsafe {
            (isatty(
                number
                    .eval_escapes()
                    .eval_variables(variables)
                    .to_string()
                    .parse()
                    .expect("Could not convert to i32"),
            ) != 0)
                .into()
        },
        TestOptions::FileExistsSocket(file) => {
            if let Ok(metadata) =
                fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                (metadata.file_type().is_socket()).into()
            } else {
                false.into()
            }
        }
        TestOptions::File1NewerThanFile2((f1, f2)) => {
            let Ok(f1_meta) = fs::metadata(f1.eval_escapes().eval_variables(variables).to_string())
            else {
                return false.into();
            };
            let Ok(f2_meta) = fs::metadata(f2.eval_escapes().eval_variables(variables).to_string())
            else {
                return false.into();
            };
            (f1_meta.modified().unwrap() > f2_meta.modified().unwrap()).into()
        }
        TestOptions::File1OlderThanFile2((f1, f2)) => !execute(
            invert,
            TestOptions::File1NewerThanFile2((f1, f2)),
            variables,
        ),
        TestOptions::File1SameAsFile2((f1, f2)) => {
            let Ok(f1_meta) = fs::metadata(f1.eval_escapes().eval_variables(variables).to_string())
            else {
                return false.into();
            };
            let Ok(f2_meta) = fs::metadata(f2.eval_escapes().eval_variables(variables).to_string())
            else {
                return false.into();
            };
            (f1_meta.ino() == f2_meta.ino()).into()
        }
        TestOptions::FileExistsOwnerEffectiveUserID(file) => {
            let uid = match fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                Ok(yay) => yay.uid(),
                Err(_) => return false.into(),
            };
            let current_uid = unsafe { libc::geteuid() };

            (uid == current_uid).into()
        }
        TestOptions::FileExistsOwnerEffectiveGroupID(file) => {
            let gid = match fs::metadata(file.eval_escapes().eval_variables(variables).to_string())
            {
                Ok(yay) => yay.gid(),
                Err(_) => return false.into(),
            };
            let current_gid = unsafe { libc::getegid() };

            (gid == current_gid).into()
        }
    };
    if invert {
        !ret
    } else {
        ret
    }
}
