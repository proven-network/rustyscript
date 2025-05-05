use std::borrow::Cow;
use std::path::Path;

use super::{web::PermissionsContainer, ExtensionTrait};
use deno_core::{extension, Extension};
use deno_fs::{CheckedPath, FileSystemRc, GetPath};
use deno_io::fs::FsError;
use deno_permissions::{PermissionCheckError, PermissionDeniedError};

extension!(
    init_fs,
    deps = [rustyscript],
    esm_entry_point = "ext:init_fs/init_fs.js",
    esm = [ dir "src/ext/fs", "init_fs.js" ],
);
impl ExtensionTrait<()> for init_fs {
    fn init((): ()) -> Extension {
        init_fs::init()
    }
}
impl ExtensionTrait<FileSystemRc> for deno_fs::deno_fs {
    fn init(fs: FileSystemRc) -> Extension {
        deno_fs::deno_fs::init::<PermissionsContainer>(fs)
    }
}

pub fn extensions(fs: FileSystemRc, is_snapshot: bool) -> Vec<Extension> {
    vec![
        deno_fs::deno_fs::build(fs, is_snapshot),
        init_fs::build((), is_snapshot),
    ]
}

impl deno_fs::FsPermissions for PermissionsContainer {
    fn check_open<'a>(
        &mut self,
        read: bool,
        write: bool,
        path: Cow<'a, std::path::Path>,
        api_name: &str,
        get_path: &'a dyn GetPath,
    ) -> Result<CheckedPath<'a>, FsError> {
        self.0.check_open(read, write, path, api_name, get_path)
    }

    fn check_read(
        &mut self,
        path: &str,
        api_name: &str,
    ) -> Result<std::path::PathBuf, PermissionCheckError> {
        self.0.check_read_all(Some(api_name))?;
        match self.0.check_read(Path::new(path), Some(api_name)) {
            Ok(cow) => Ok(cow.into_owned()),
            Err(_) => Err(PermissionCheckError::PermissionDenied(
                PermissionDeniedError::Fatal {
                    access: "read access".to_string(),
                },
            )),
        }
    }

    fn check_read_path<'a>(
        &mut self,
        path: std::borrow::Cow<'a, std::path::Path>,
        api_name: &str,
    ) -> Result<std::borrow::Cow<'a, std::path::Path>, PermissionCheckError> {
        self.0.check_read_all(Some(api_name))?;
        match self.0.check_read(&path, Some(api_name)) {
            Ok(cow) => Ok(std::borrow::Cow::Owned(cow.into_owned())),
            Err(denied_err) => Err(PermissionCheckError::PermissionDenied(denied_err)),
        }
    }

    fn check_read_all(&mut self, api_name: &str) -> Result<(), PermissionCheckError> {
        self.0.check_read_all(Some(api_name))?;
        Ok(())
    }

    fn check_read_blind(
        &mut self,
        p: &std::path::Path,
        display: &str,
        api_name: &str,
    ) -> Result<(), PermissionCheckError> {
        self.0.check_read_all(Some(api_name))?;
        self.0.check_read_blind(p, display, api_name)?;
        Ok(())
    }

    fn check_write(
        &mut self,
        path: &str,
        api_name: &str,
    ) -> Result<std::path::PathBuf, PermissionCheckError> {
        self.0.check_write_all(api_name)?;
        match self.0.check_write(Path::new(path), Some(api_name)) {
            Ok(cow) => Ok(cow.into_owned()),
            Err(_) => Err(PermissionCheckError::PermissionDenied(
                PermissionDeniedError::Fatal {
                    access: "write access".to_string(),
                },
            )),
        }
    }

    fn check_write_path<'a>(
        &mut self,
        path: std::borrow::Cow<'a, std::path::Path>,
        api_name: &str,
    ) -> Result<std::borrow::Cow<'a, std::path::Path>, PermissionCheckError> {
        self.0.check_write_all(api_name)?;
        match self.0.check_write(&path, Some(api_name)) {
            Ok(cow) => Ok(std::borrow::Cow::Owned(cow.into_owned())),
            Err(_) => Err(PermissionCheckError::PermissionDenied(
                PermissionDeniedError::Fatal {
                    access: "write access".to_string(),
                },
            )),
        }
    }

    fn check_write_partial(
        &mut self,
        path: &str,
        api_name: &str,
    ) -> Result<std::path::PathBuf, PermissionCheckError> {
        self.0.check_write_all(api_name)?;
        match self.0.check_write_partial(path, api_name) {
            Ok(p) => Ok(p),
            Err(_) => Err(PermissionCheckError::PermissionDenied(
                PermissionDeniedError::Fatal {
                    access: "write access".to_string(),
                },
            )),
        }
    }

    fn check_write_all(&mut self, api_name: &str) -> Result<(), PermissionCheckError> {
        self.0.check_write_all(api_name)?;
        Ok(())
    }

    fn check_write_blind(
        &mut self,
        p: &std::path::Path,
        display: &str,
        api_name: &str,
    ) -> Result<(), PermissionCheckError> {
        self.0.check_write_all(api_name)?;
        self.0.check_write_blind(p, display, api_name)?;
        Ok(())
    }
}
