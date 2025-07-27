use deno_fs::FsError;
use deno_permissions::{CheckedPath, OpenAccessKind};
use deno_permissions::PermissionCheckError;
use std::{
    borrow::Cow,
    collections::HashSet,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

pub use deno_permissions::PermissionDeniedError;

pub fn oops<T>(msg: impl std::fmt::Display) -> Result<T, PermissionDeniedError> {
    Err(PermissionDeniedError {
        access: msg.to_string(),
        name: "permission",
    })
}

/// The default permissions manager for the web related extensions
///
/// Allows all operations
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultWebPermissions;
impl WebPermissions for DefaultWebPermissions {
    fn allow_hrtime(&self) -> bool {
        true
    }

    fn check_url(
        &self,
        url: &deno_core::url::Url,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        Ok(())
    }

    fn check_open<'a>(
        &self,
        path: Cow<'a, Path>,
        _access_kind: OpenAccessKind,
        _api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        Ok(CheckedPath::unsafe_new(path))
    }

    fn check_open_blind<'a>(
        &self,
        path: Cow<'a, Path>,
        _access_kind: OpenAccessKind,
        _display: &str,
        _api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        Ok(CheckedPath::unsafe_new(path))
    }

    fn check_read_path<'a>(
        &self,
        p: Cow<'a, Path>,
        _api_name: Option<&str>,
    ) -> Result<CheckedPath<'a>, FsError> {
        Ok(CheckedPath::unsafe_new(p))
    }

    fn check_read<'a>(
        &self,
        p: &'a Path,
        api_name: Option<&str>,
    ) -> Result<Cow<'a, Path>, PermissionDeniedError> {
        Ok(Cow::Borrowed(p))
    }

    fn check_read_all(&self, api_name: &str) -> Result<(), PermissionCheckError> {
        Ok(())
    }

    fn check_read_blind(
        &self,
        p: &Path,
        display: &str,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        Ok(())
    }

    fn check_write<'a>(
        &self,
        p: &'a Path,
        api_name: Option<&str>,
    ) -> Result<Cow<'a, Path>, PermissionDeniedError> {
        Ok(Cow::Borrowed(p))
    }

    fn check_write_all(&self, api_name: &str) -> Result<(), PermissionCheckError> {
        Ok(())
    }

    fn check_write_blind(
        &self,
        p: &Path,
        display: &str,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        Ok(())
    }

    fn check_write_partial<'a>(
        &self,
        path: Cow<'a, Path>,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        Ok(CheckedPath::unsafe_new(path))
    }

    fn check_host(
        &self,
        host: &str,
        port: Option<u16>,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        Ok(())
    }

    fn check_sys(
        &self,
        kind: SystemsPermissionKind,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        Ok(())
    }

    fn check_env(&self, var: &str) -> Result<(), PermissionDeniedError> {
        Ok(())
    }

    fn check_exec(&self) -> Result<(), PermissionDeniedError> {
        Ok(())
    }
}

// Inner container for the allowlist permission set
#[derive(Clone, Default, Debug)]
#[allow(clippy::struct_excessive_bools)]
struct AllowlistWebPermissionsSet {
    pub hrtime: bool,
    pub exec: bool,
    pub read_all: bool,
    pub write_all: bool,
    pub url: HashSet<String>,
    pub openr_paths: HashSet<String>,
    pub openw_paths: HashSet<String>,
    pub envs: HashSet<String>,
    pub sys: HashSet<SystemsPermissionKind>,
    pub read_paths: HashSet<String>,
    pub write_paths: HashSet<String>,
    pub hosts: HashSet<String>,
}

/// Permissions manager for the web related extensions
///
/// Allows only operations that are explicitly enabled
///
/// Uses interior mutability to allow changing the permissions at runtime
#[derive(Clone, Default, Debug)]
pub struct AllowlistWebPermissions(Arc<RwLock<AllowlistWebPermissionsSet>>);
impl AllowlistWebPermissions {
    /// Create a new instance with nothing allowed by default
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(AllowlistWebPermissionsSet::default())))
    }

    fn borrow(&self) -> std::sync::RwLockReadGuard<AllowlistWebPermissionsSet> {
        self.0.read().expect("Could not lock permissions")
    }

    fn borrow_mut(&self) -> std::sync::RwLockWriteGuard<AllowlistWebPermissionsSet> {
        self.0.write().expect("Could not lock permissions")
    }

    /// Set the `hrtime` permission
    ///
    /// If true, timers will be allowed to use high resolution time
    pub fn set_hrtime(&self, value: bool) {
        self.borrow_mut().hrtime = value;
    }

    /// Set the `exec` permission
    ///
    /// If true, FFI execution will be allowed
    pub fn set_exec(&self, value: bool) {
        self.borrow_mut().exec = value;
    }

    /// Set the `read_all` permission
    ///
    /// If false all reads will be denied
    pub fn set_read_all(&self, value: bool) {
        self.borrow_mut().read_all = value;
    }

    /// Set the `write_all` permission
    ///
    /// If false all writes will be denied
    pub fn set_write_all(&self, value: bool) {
        self.borrow_mut().write_all = value;
    }

    /// Whitelist a path for opening
    ///
    /// If `read` is true, the path will be allowed to be opened for reading  
    /// If `write` is true, the path will be allowed to be opened for writing
    pub fn allow_open(&self, path: &str, read: bool, write: bool) {
        if read {
            self.borrow_mut().openr_paths.insert(path.to_string());
        }
        if write {
            self.borrow_mut().openw_paths.insert(path.to_string());
        }
    }

    /// Whitelist a URL
    pub fn allow_url(&self, url: &str) {
        self.borrow_mut().url.insert(url.to_string());
    }

    /// Blacklist a URL
    pub fn deny_url(&self, url: &str) {
        self.borrow_mut().url.remove(url);
    }

    /// Whitelist a path for reading
    pub fn allow_read(&self, path: &str) {
        self.borrow_mut().read_paths.insert(path.to_string());
    }

    /// Blacklist a path for reading
    pub fn deny_read(&self, path: &str) {
        self.borrow_mut().read_paths.remove(path);
    }

    /// Whitelist a path for writing
    pub fn allow_write(&self, path: &str) {
        self.borrow_mut().write_paths.insert(path.to_string());
    }

    /// Blacklist a path for writing
    pub fn deny_write(&self, path: &str) {
        self.borrow_mut().write_paths.remove(path);
    }

    /// Whitelist a host
    pub fn allow_host(&self, host: &str) {
        self.borrow_mut().hosts.insert(host.to_string());
    }

    /// Blacklist a host
    pub fn deny_host(&self, host: &str) {
        self.borrow_mut().hosts.remove(host);
    }

    /// Whitelist an environment variable
    pub fn allow_env(&self, var: &str) {
        self.borrow_mut().envs.insert(var.to_string());
    }

    /// Blacklist an environment variable
    pub fn deny_env(&self, var: &str) {
        self.borrow_mut().envs.remove(var);
    }

    /// Whitelist a system operation
    pub fn allow_sys(&self, kind: SystemsPermissionKind) {
        self.borrow_mut().sys.insert(kind);
    }

    /// Blacklist a system operation
    pub fn deny_sys(&self, kind: SystemsPermissionKind) {
        self.borrow_mut().sys.remove(&kind);
    }
}
impl WebPermissions for AllowlistWebPermissions {
    fn allow_hrtime(&self) -> bool {
        self.borrow().hrtime
    }

    fn check_host(
        &self,
        host: &str,
        port: Option<u16>,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        if self.borrow().hosts.contains(host) {
            Ok(())
        } else {
            oops(host)?
        }
    }

    fn check_url(
        &self,
        url: &deno_core::url::Url,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        if self.borrow().url.contains(url.as_str()) {
            Ok(())
        } else {
            oops(url)?
        }
    }

    fn check_read_path<'a>(
        &self,
        p: Cow<'a, Path>,
        _api_name: Option<&str>,
    ) -> Result<CheckedPath<'a>, FsError> {
        let inst = self.borrow();
        if !inst.read_all {
            let _msg = oops::<()>(format!("read access denied for {}", p.display()))
                .unwrap_err()
                .to_string();
            return Err(FsError::PermissionCheck(PermissionCheckError::PermissionDenied(
                PermissionDeniedError {
                    access: "read access denied".to_string(),
                    name: "read",
                }
            )));
        }
        if !inst.read_paths.contains(p.to_str().unwrap()) {
            let _msg = oops::<()>(format!(
                "read access denied for {}",
                p.display()
            ))
            .unwrap_err()
            .to_string();
            return Err(FsError::PermissionCheck(PermissionCheckError::PermissionDenied(
                PermissionDeniedError {
                    access: "read access denied".to_string(),
                    name: "read",
                }
            )));
        }
        Ok(CheckedPath::unsafe_new(p))
    }

    fn check_read<'a>(
        &self,
        p: &'a Path,
        api_name: Option<&str>,
    ) -> Result<Cow<'a, Path>, PermissionDeniedError> {
        let inst = self.borrow();
        if inst.read_all && inst.read_paths.contains(p.to_str().unwrap()) {
            Ok(Cow::Borrowed(p))
        } else {
            oops(p.display())?
        }
    }

    fn check_write<'a>(
        &self,
        p: &'a Path,
        api_name: Option<&str>,
    ) -> Result<Cow<'a, Path>, PermissionDeniedError> {
        let inst = self.borrow();
        if inst.write_all && inst.write_paths.contains(p.to_str().unwrap()) {
            Ok(Cow::Borrowed(p))
        } else {
            oops(p.display())?
        }
    }

    fn check_open<'a>(
        &self,
        path: Cow<'a, Path>,
        access_kind: OpenAccessKind,
        _api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        let inst = self.borrow();
        let path_str = path.to_str().ok_or_else(|| {
            PermissionCheckError::PermissionDenied(
                PermissionDeniedError {
                    access: "invalid filename".to_string(),
                    name: "open",
                }
            )
        })?;

        // Check permissions based on access kind
        match access_kind {
            OpenAccessKind::Read | OpenAccessKind::ReadNoFollow => {
                if !inst.openr_paths.contains(path_str) {
                    return Err(PermissionCheckError::PermissionDenied(
                        PermissionDeniedError {
                            access: "open read denied".to_string(),
                            name: "read",
                        }
                    ));
                }
            }
            OpenAccessKind::Write | OpenAccessKind::WriteNoFollow => {
                if !inst.openw_paths.contains(path_str) {
                    return Err(PermissionCheckError::PermissionDenied(
                        PermissionDeniedError {
                            access: "open write denied".to_string(),
                            name: "write",
                        }
                    ));
                }
            }
            OpenAccessKind::ReadWrite | OpenAccessKind::ReadWriteNoFollow => {
                if !inst.openr_paths.contains(path_str) || !inst.openw_paths.contains(path_str) {
                    return Err(PermissionCheckError::PermissionDenied(
                        PermissionDeniedError {
                            access: "open read/write denied".to_string(),
                            name: "write",
                        }
                    ));
                }
            }
        }

        Ok(CheckedPath::unsafe_new(path))
    }

    fn check_open_blind<'a>(
        &self,
        path: Cow<'a, Path>,
        access_kind: OpenAccessKind,
        _display: &str,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        self.check_open(path, access_kind, api_name)
    }

    fn check_read_all(&self, api_name: &str) -> Result<(), PermissionCheckError> {
        if self.borrow().read_all {
            Ok(())
        } else {
            Err(PermissionCheckError::PermissionDenied(
                PermissionDeniedError {
                    access: "read_all".to_string(),
                    name: "read",
                }
            ))
        }
    }

    fn check_read_blind(
        &self,
        p: &Path,
        display: &str,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        let inst = self.borrow();
        if !inst.read_all {
            return oops("read_all")?;
        }
        if !inst.read_paths.contains(p.to_str().unwrap()) {
            return oops(p.display())?;
        }
        Ok(())
    }

    fn check_write_all(&self, api_name: &str) -> Result<(), PermissionCheckError> {
        if self.borrow().write_all {
            Ok(())
        } else {
            Err(PermissionCheckError::PermissionDenied(
                PermissionDeniedError {
                    access: "write_all".to_string(),
                    name: "write",
                }
            ))
        }
    }

    fn check_write_blind(
        &self,
        p: &Path,
        display: &str,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        self.check_write(Path::new(p), Some(api_name))?;
        Ok(())
    }

    fn check_write_partial<'a>(
        &self,
        path: Cow<'a, Path>,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        let inst = self.borrow();
        if !inst.write_all || !inst.write_paths.contains(path.to_str().unwrap()) {
            return Err(PermissionCheckError::PermissionDenied(
                PermissionDeniedError {
                    access: "write partial denied".to_string(),
                    name: "write",
                }
            ));
        }
        Ok(CheckedPath::unsafe_new(path))
    }

    fn check_sys(
        &self,
        kind: SystemsPermissionKind,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError> {
        if self.borrow().sys.contains(&kind) {
            Ok(())
        } else {
            oops(kind.as_str())?
        }
    }

    fn check_env(&self, var: &str) -> Result<(), PermissionDeniedError> {
        if self.borrow().envs.contains(var) {
            Ok(())
        } else {
            oops(var)?
        }
    }

    fn check_exec(&self) -> Result<(), PermissionDeniedError> {
        if self.borrow().exec {
            Ok(())
        } else {
            oops("ffi")?
        }
    }
}

/// Trait managing the permissions for the web related extensions
///
/// See [`DefaultWebPermissions`] for a default implementation that allows-all
pub trait WebPermissions: std::fmt::Debug + Send + Sync {
    /// Check if `hrtime` is allowed
    ///
    /// If true, timers will be allowed to use high resolution time
    fn allow_hrtime(&self) -> bool;

    /// Check if a URL is allowed to be used by fetch or websocket
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_url(
        &self,
        url: &deno_core::url::Url,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError>;

    /// Check if a path is allowed to be opened by fs
    /// Corresponds to deno_fs::FsPermissions::check_open
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_open<'a>(
        &self,
        path: Cow<'a, Path>,
        access_kind: OpenAccessKind,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError>;

    /// Check if a path is allowed to be opened by fs blind
    /// Corresponds to deno_fs::FsPermissions::check_open_blind
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_open_blind<'a>(
        &self,
        path: Cow<'a, Path>,
        access_kind: OpenAccessKind,
        display: &str,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError>;

    /// Check if a path is allowed to be read by fetch or net
    /// Corresponds to deno_fetch::FetchPermissions::check_read
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_read_path<'a>(
        &self,
        p: Cow<'a, Path>,
        api_name: Option<&str>,
    ) -> Result<CheckedPath<'a>, FsError>;

    /// Check if a path is allowed to be read (without GetPath)
    /// Corresponds to deno_net::NetPermissions::check_read and deno_fs::FsPermissions::check_read
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_read<'a>(
        &self,
        p: &'a Path,
        api_name: Option<&str>,
    ) -> Result<Cow<'a, Path>, PermissionDeniedError>;

    /// Check if all paths are allowed to be read by fs
    ///
    /// Used by `deno_fs` for `op_fs_symlink`
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_read_all(&self, api_name: &str) -> Result<(), PermissionCheckError>;

    /// Check if a path is allowed to be read by fs
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_read_blind(
        &self,
        p: &Path,
        display: &str,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError>;

    /// Check if a path is allowed to be written to by net
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_write<'a>(
        &self,
        p: &'a Path,
        api_name: Option<&str>,
    ) -> Result<Cow<'a, Path>, PermissionDeniedError>;

    /// Check if all paths are allowed to be written to by fs
    ///
    /// Used by `deno_fs` for `op_fs_symlink`
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_write_all(&self, api_name: &str) -> Result<(), PermissionCheckError>;

    /// Check if a path is allowed to be written to by fs
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_write_blind(
        &self,
        p: &Path,
        display: &str,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError>;

    /// Check if a path is allowed to be written to by fs
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_write_partial<'a>(
        &self,
        path: Cow<'a, Path>,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError>;

    /// Check if a host is allowed to be connected to by net
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_host(
        &self,
        host: &str,
        port: Option<u16>,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError>;

    /// Check if a system operation is allowed
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_sys(
        &self,
        kind: SystemsPermissionKind,
        api_name: &str,
    ) -> Result<(), PermissionDeniedError>;

    /// Check if an environment variable is allowed to be accessed
    ///
    /// Used by remote KV store (`deno_kv`)
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_env(&self, var: &str) -> Result<(), PermissionDeniedError>;

    /// Check if FFI execution is allowed
    ///
    /// # Errors
    /// If an error is returned, the operation will be denied with the error message as the reason
    fn check_exec(&self) -> Result<(), PermissionDeniedError>;
}

macro_rules! impl_sys_permission_kinds {
    ($($kind:ident($name:literal)),+ $(,)?) => {
        /// Knows systems permission checks performed by deno
        ///
        /// This list is updated manually using:
        /// <https://github.com/search?q=repo%3Adenoland%2Fdeno+check_sys%28%22&type=code>
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum SystemsPermissionKind {
            $(
                #[doc = stringify!($kind)]
                $kind,
            )+

            /// A custom permission kind
            Other(String),
        }
        impl SystemsPermissionKind {
            /// Create a new instance from a string
            #[must_use]
            pub fn new(s: &str) -> Self {
                match s {
                    $( $name => Self::$kind, )+
                    _ => Self::Other(s.to_string()),
                }
            }

            /// Get the string representation of the permission
            #[must_use]
            pub fn as_str(&self) -> &str {
                match self {
                    $( Self::$kind => $name, )+
                    Self::Other(s) => &s,
                }
            }
        }
    };
}

impl_sys_permission_kinds!(
    LoadAvg("loadavg"),
    Hostname("hostname"),
    OsRelease("osRelease"),
    Networkinterfaces("networkInterfaces"),
    StatFs("statfs"),
    GetPriority("getPriority"),
    SystemMemoryInfo("systemMemoryInfo"),
    Gid("gid"),
    Uid("uid"),
    OsUptime("osUptime"),
    SetPriority("setPriority"),
    UserInfo("userInfo"),
    GetEGid("getegid"),
    Cpus("cpus"),
    HomeDir("homeDir"),
    Inspector("inspector"),
);

#[derive(Clone, Debug)]
pub struct PermissionsContainer(pub Arc<dyn WebPermissions>);
impl deno_web::TimersPermission for PermissionsContainer {
    fn allow_hrtime(&mut self) -> bool {
        self.0.allow_hrtime()
    }
}
impl deno_fetch::FetchPermissions for PermissionsContainer {
    fn check_net_url(
        &mut self,
        url: &reqwest::Url,
        api_name: &str,
    ) -> Result<(), PermissionCheckError> {
        self.0.check_url(url, api_name)?;
        Ok(())
    }

    fn check_net_vsock(
        &mut self,
        cid: u32,
        port: u32,
        api_name: &str,
    ) -> Result<(), PermissionCheckError> {
        Err(PermissionCheckError::PermissionDenied(
            PermissionDeniedError {
                access: "vsock".to_string(),
                name: "net",
            },
        ))
    }

    fn check_open<'a>(
        &mut self,
        path: Cow<'a, Path>,
        access_kind: OpenAccessKind,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        self.0.check_open(path, access_kind, api_name)
    }
}
impl deno_net::NetPermissions for PermissionsContainer {
    fn check_net<T: AsRef<str>>(
        &mut self,
        host: &(T, Option<u16>),
        api_name: &str,
    ) -> Result<(), PermissionCheckError> {
        self.0.check_host(host.0.as_ref(), host.1, api_name)?;
        Ok(())
    }

    fn check_open<'a>(
        &mut self,
        path: Cow<'a, Path>,
        access_kind: OpenAccessKind,
        api_name: &str,
    ) -> Result<CheckedPath<'a>, PermissionCheckError> {
        self.0.check_open(path, access_kind, api_name)
    }

    fn check_vsock(
        &mut self,
        cid: u32,
        port: u32,
        api_name: &str,
    ) -> Result<(), PermissionCheckError> {
        Err(PermissionCheckError::PermissionDenied(
            PermissionDeniedError {
                access: "vsock".to_string(),
                name: "net",
            },
        ))
    }

}
