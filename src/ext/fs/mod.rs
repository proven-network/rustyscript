use std::borrow::Cow;
use std::path::Path;

use super::{web::PermissionsContainer, ExtensionTrait};
use deno_core::{extension, Extension};
use deno_fs::FileSystemRc;
use deno_io::fs::FsError;
use deno_permissions::{CheckedPath, OpenAccessKind};
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
        &self,
        path: Cow<'a, Path>,
        access_kind: OpenAccessKind,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        self.0.check_open(path, access_kind, api_name)
    }

    fn check_open_blind<'a>(
        &self,
        path: Cow<'a, Path>,
        access_kind: OpenAccessKind,
        display: &str,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        self.0
            .check_open_blind(path, access_kind, display, api_name)
    }

    fn check_read_all(&self, api_name: &str) -> Result<(), PermissionCheckError> {
        self.0.check_read_all(api_name)
    }

    fn check_write_partial<'a>(
        &self,
        path: Cow<'a, Path>,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        self.0.check_write_partial(path, api_name)
    }

    fn check_write_all(&self, api_name: &str) -> Result<(), PermissionCheckError> {
        self.0.check_write_all(api_name)
    }
}
