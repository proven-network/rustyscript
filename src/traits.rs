use crate::Error;
use deno_core::v8::{self, HandleScope};
use deno_core::ModuleSpecifier;
use deno_path_util::PathToUrlError;
use std::path::Path;

/// Converts a string representing a relative or absolute path into a
/// `ModuleSpecifier`. A relative path is considered relative to the passed
/// `current_dir`.
///
/// This is a patch for the str only `deno_core` provided version
fn resolve_path(
    path_str: impl AsRef<Path>,
    current_dir: &Path,
) -> Result<ModuleSpecifier, PathToUrlError> {
    let path = current_dir.join(path_str);
    let path = deno_core::normalize_path(path);

    deno_path_util::url_from_file_path(&path)
}

pub trait ToModuleSpecifier {
    fn to_module_specifier(&self, base: &Path) -> Result<ModuleSpecifier, Error>;
}

impl<T: AsRef<Path>> ToModuleSpecifier for T {
    fn to_module_specifier(&self, base: &Path) -> Result<ModuleSpecifier, Error> {
        resolve_path(self, base).map_err(|e| Error::ModuleNotFound(e.to_string()))
    }
}

pub trait ToV8String {
    fn to_v8_string<'a>(
        &self,
        scope: &mut HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::String>, Error>;
}

impl ToV8String for str {
    fn to_v8_string<'a>(
        &self,
        scope: &mut HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::String>, Error> {
        v8::String::new(scope, self).ok_or(Error::V8Encoding(self.to_string()))
    }
}

pub trait ToDefinedValue<T> {
    fn if_defined(&self) -> Option<T>;
}

impl<'a> ToDefinedValue<v8::Local<'a, v8::Value>> for Option<v8::Local<'a, v8::Value>> {
    fn if_defined(&self) -> Option<v8::Local<'a, v8::Value>> {
        self.filter(|v| !v.is_undefined())
    }
}
