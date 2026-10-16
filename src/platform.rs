//! Explicit platform interface for Luca Code (`Platform`).
//!
//! The Luca runtime requests operating-system capabilities through this
//! trait; the host decides how they are provided. `NativePlatform` delegates
//! to the standard library exactly as the interpreter historically did, and
//! `FakePlatform` is deterministic per-instance test support.
//!
//! M1b-1 introduces this interface only. No runtime call site uses it yet,
//! so observable behavior is unchanged. See `docs/implementation-status.md`
//! (M1b-1). Later milestones migrate call sites one group at a time.

use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    rc::Rc,
};

/// Machine-readable category of a [`PlatformError`].
///
/// Compatibility contract: [`PlatformError::from_io`] maps
/// `io::ErrorKind::NotFound` to `NotFound` and `PermissionDenied` to
/// `PermissionDenied`; every other I/O kind maps to `Other`.
/// `InvalidInput` and `Unsupported` are never produced from I/O errors;
/// hosts and fakes construct them explicitly for scripted or
/// capability-denied operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformErrorKind {
    NotFound,
    PermissionDenied,
    InvalidInput,
    Unsupported,
    Other,
}

/// A platform operation failure.
///
/// Compatibility contract: `detail` preserves the complete source error
/// text. For errors built by [`PlatformError::from_io`], `detail` is the
/// full `io::Error` Display output, including OS error information
/// (e.g. `No such file or directory (os error 2)`), so existing Luca
/// diagnostics that embed `{e}` render byte-identically. [`fmt::Display`]
/// for this type yields `detail` unchanged for the same reason. Fakes
/// construct deterministic `detail` strings that are never compared
/// against host OS text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformError {
    pub kind: PlatformErrorKind,
    pub detail: String,
}

impl PlatformError {
    pub fn new(kind: PlatformErrorKind, detail: impl Into<String>) -> Self {
        Self { kind, detail: detail.into() }
    }

    /// Build from a standard I/O error, preserving kind (where mapped) and
    /// the complete Display text.
    pub fn from_io(error: io::Error) -> Self {
        let kind = match error.kind() {
            io::ErrorKind::NotFound => PlatformErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => PlatformErrorKind::PermissionDenied,
            _ => PlatformErrorKind::Other,
        };
        Self { kind, detail: error.to_string() }
    }

    /// Build an explicitly denied capability (hosts without the operation,
    /// scripted fake denials). Never produced by [`PlatformError::from_io`].
    pub fn unsupported(detail: impl Into<String>) -> Self {
        Self { kind: PlatformErrorKind::Unsupported, detail: detail.into() }
    }
}

impl fmt::Display for PlatformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.detail)
    }
}

impl std::error::Error for PlatformError {}

/// Operating-system capabilities requested by the Luca runtime.
///
/// Every method documents the historical `std::` call it mirrors so later
/// migrations preserve behavior exactly. Methods take `&self`: native
/// implementations are stateless, and fakes use interior mutability, so a
/// single boxed trait object can serve a whole interpreter run with no
/// global state.
pub trait Platform {
    /// Read a whole file as a string. Mirrors `std::fs::read_to_string`.
    /// Native maps failures via [`PlatformError::from_io`].
    fn read_file(&self, path: &Path) -> Result<String, PlatformError>;

    /// Overwrite (or create) a file with `content`. Mirrors `std::fs::write`.
    fn write_file(&self, path: &Path, content: &str) -> Result<(), PlatformError>;

    /// Append `content` to a file, creating it when absent. Mirrors
    /// `OpenOptions::new().create(true).append(true).open()` plus
    /// `write_all`, as used by `File.append`.
    fn append_file(&self, path: &Path, content: &str) -> Result<(), PlatformError>;

    /// Delete a file. Mirrors `std::fs::remove_file`.
    fn remove_file(&self, path: &Path) -> Result<(), PlatformError>;

    /// Report whether a path exists. Mirrors `Path::exists`, including its
    /// contract that every filesystem error maps to `false` (never an
    /// error). Readability failures therefore surface at read time, not
    /// here; do not change this lossy-probe contract without approval.
    /// Covers files and directories alike, exactly like `Path::exists`.
    fn path_exists(&self, path: &Path) -> bool;

    /// Report whether a path is a regular file. Mirrors `Path::is_file`,
    /// including its contract that every filesystem error maps to `false`
    /// (never an error). Module resolution depends on this distinction: a
    /// directory named `Example.lucc` must not count as a module file, so
    /// resolvers must call `is_file`, never `path_exists`, for candidacy.
    fn is_file(&self, path: &Path) -> bool;

    /// Emit one runtime output line. Mirrors `println!` as used by the
    /// interpreter's output flush. Infallible by contract: hosts sink the
    /// line as today (standard output for native); a broken sink behaves
    /// exactly as `println!` does today.
    fn emit_output(&self, line: &str);

    /// Emit one `Warning at line:column: message` warning. Mirrors the
    /// interpreter's `warn_at` (`eprintln!`). Infallible by contract, same
    /// rationale as [`Platform::emit_output`].
    fn emit_warning(&self, line: usize, column: usize, message: &str);

    /// Read one input line for `ask`, without trailing `\n`/`\r`.
    /// Mirrors the stdin branch of the interpreter's `read_input_line`
    /// (BufRead `read_line` plus newline stripping). End of input is
    /// reported as an error (native detail `No more input available`,
    /// matching the historical Luca-level message); I/O failures map via
    /// [`PlatformError::from_io`].
    fn read_line(&self) -> Result<String, PlatformError>;

    /// Process environment as unsorted key/value pairs. Mirrors
    /// `std::env::vars`; sorting stays with the caller (`System.env`
    /// sorts by key), so implementations must not sort.
    fn env_vars(&self) -> Vec<(String, String)>;

    /// Operating system name. Mirrors `std::env::consts::OS`.
    fn os(&self) -> &'static str;

    /// Processor architecture name. Mirrors `std::env::consts::ARCH`.
    fn arch(&self) -> &'static str;

    /// Working directory. Mirrors `std::env::current_dir`, mapping
    /// failures via [`PlatformError::from_io`]. Callers keep their
    /// historical silent fallbacks; do not revise fallback policy here.
    fn current_dir(&self) -> Result<PathBuf, PlatformError>;
}

/// Production implementation: delegates to the standard library exactly
/// as the interpreter historically did. No behavior change.
pub struct NativePlatform;

impl Platform for NativePlatform {
    fn read_file(&self, path: &Path) -> Result<String, PlatformError> {
        std::fs::read_to_string(path).map_err(PlatformError::from_io)
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), PlatformError> {
        std::fs::write(path, content).map_err(PlatformError::from_io)
    }

    fn append_file(&self, path: &Path, content: &str) -> Result<(), PlatformError> {
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path).map_err(PlatformError::from_io)?;
        file.write_all(content.as_bytes()).map_err(PlatformError::from_io)
    }

    fn remove_file(&self, path: &Path) -> Result<(), PlatformError> {
        std::fs::remove_file(path).map_err(PlatformError::from_io)
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn emit_output(&self, line: &str) {
        println!("{line}");
    }

    fn emit_warning(&self, line: usize, column: usize, message: &str) {
        eprintln!("Warning at {line}:{column}: {message}");
    }

    fn read_line(&self) -> Result<String, PlatformError> {
        let mut buf = String::new();
        match std::io::stdin().lock().read_line(&mut buf) {
            Ok(0) => Err(PlatformError::new(PlatformErrorKind::Other, "No more input available")),
            Ok(_) => {
                while buf.ends_with('\n') || buf.ends_with('\r') {
                    buf.pop();
                }
                Ok(buf)
            }
            Err(error) => Err(PlatformError::from_io(error)),
        }
    }

    fn env_vars(&self) -> Vec<(String, String)> {
        std::env::vars().collect()
    }

    fn os(&self) -> &'static str {
        std::env::consts::OS
    }

    fn arch(&self) -> &'static str {
        std::env::consts::ARCH
    }

    fn current_dir(&self) -> Result<PathBuf, PlatformError> {
        std::env::current_dir().map_err(PlatformError::from_io)
    }
}

/// Deterministic test support. Each instance owns its state (scripted
/// files and errors, environment, input queue, captured output/warnings,
/// OS/architecture labels, working-directory result); no global or shared
/// storage, so parallel tests cannot interfere.
///
/// This is intentionally a small scripted fake, not a generalized mocking
/// framework. Unscripted file reads fail deterministically with a
/// `NotFound` error whose detail names the path; an empty input queue with
/// no injected error reports end of input, mirroring the native contract.
///
/// Clones share state: `Clone` duplicates only the handle, so a clone
/// observes everything scripted into or captured by its siblings.
/// Independently created instances (separate `new()` calls) remain fully
/// isolated. There is no global or shared-across-instances storage.
#[derive(Clone)]
pub struct FakePlatform {
    state: Rc<FakeState>,
}

struct FakeState {
    files: RefCell<HashMap<PathBuf, String>>,
    dirs: RefCell<HashSet<PathBuf>>,
    file_errors: RefCell<HashMap<PathBuf, PlatformError>>,
    output: RefCell<Vec<String>>,
    warnings: RefCell<Vec<(usize, usize, String)>>,
    input: RefCell<VecDeque<String>>,
    input_error: RefCell<Option<PlatformError>>,
    env: RefCell<Vec<(String, String)>>,
    os: Cell<&'static str>,
    arch: Cell<&'static str>,
    current_dir: RefCell<Result<PathBuf, PlatformError>>,
}

impl FakePlatform {
    pub fn new() -> Self {
        Self {
            state: Rc::new(FakeState {
                files: RefCell::new(HashMap::new()),
                dirs: RefCell::new(HashSet::new()),
                file_errors: RefCell::new(HashMap::new()),
                output: RefCell::new(Vec::new()),
                warnings: RefCell::new(Vec::new()),
                input: RefCell::new(VecDeque::new()),
                input_error: RefCell::new(None),
                env: RefCell::new(Vec::new()),
                os: Cell::new(std::env::consts::OS),
                arch: Cell::new(std::env::consts::ARCH),
                current_dir: RefCell::new(Ok(PathBuf::from("/fake/work"))),
            }),
        }
    }

    /// Script file contents returned by `read_file`.
    pub fn add_file(&self, path: impl AsRef<Path>, content: impl Into<String>) {
        self.state.files.borrow_mut().insert(path.as_ref().to_path_buf(), content.into());
    }

    /// Script a directory. Directories satisfy `path_exists` but never
    /// `is_file`, mirroring `Path` semantics, so module resolution skips
    /// them while existence probes still report them.
    pub fn add_dir(&self, path: impl AsRef<Path>) {
        self.state.dirs.borrow_mut().insert(path.as_ref().to_path_buf());
    }

    /// Script a filesystem error returned (instead of storage access) by
    /// `read_file`, `write_file`, `append_file`, and `remove_file`.
    pub fn fail_file(&self, path: impl AsRef<Path>, error: PlatformError) {
        self.state.file_errors.borrow_mut().insert(path.as_ref().to_path_buf(), error);
    }

    /// Script environment variables returned by `env_vars`.
    pub fn set_env(&self, vars: Vec<(String, String)>) {
        *self.state.env.borrow_mut() = vars;
    }

    /// Queue one input line returned by `read_line`.
    pub fn push_input(&self, line: impl Into<String>) {
        self.state.input.borrow_mut().push_back(line.into());
    }

    /// Replace the whole input queue.
    pub fn set_input(&self, lines: Vec<String>) {
        *self.state.input.borrow_mut() = lines.into();
    }

    /// Script an input error returned by `read_line` while set.
    pub fn fail_input(&self, error: PlatformError) {
        *self.state.input_error.borrow_mut() = Some(error);
    }

    /// Clear a scripted input error.
    pub fn clear_input_error(&self) {
        *self.state.input_error.borrow_mut() = None;
    }

    /// Override the reported OS and architecture labels.
    pub fn set_os_arch(&self, os: &'static str, arch: &'static str) {
        self.state.os.set(os);
        self.state.arch.set(arch);
    }

    /// Script the `current_dir` result, including failure.
    pub fn set_current_dir(&self, result: Result<PathBuf, PlatformError>) {
        *self.state.current_dir.borrow_mut() = result;
    }

    /// Lines passed to `emit_output`, in order.
    pub fn output(&self) -> Vec<String> {
        self.state.output.borrow().clone()
    }

    /// Warnings passed to `emit_warning`, in order.
    pub fn warnings(&self) -> Vec<(usize, usize, String)> {
        self.state.warnings.borrow().clone()
    }
}

impl Default for FakePlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for FakePlatform {
    fn read_file(&self, path: &Path) -> Result<String, PlatformError> {
        if let Some(error) = self.state.file_errors.borrow().get(path) {
            return Err(error.clone());
        }
        self.state.files.borrow().get(path).cloned().ok_or_else(|| {
            PlatformError::new(PlatformErrorKind::NotFound, format!("No such file: {}", path.display()))
        })
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), PlatformError> {
        if let Some(error) = self.state.file_errors.borrow().get(path) {
            return Err(error.clone());
        }
        self.state.files.borrow_mut().insert(path.to_path_buf(), content.to_owned());
        Ok(())
    }

    fn append_file(&self, path: &Path, content: &str) -> Result<(), PlatformError> {
        if let Some(error) = self.state.file_errors.borrow().get(path) {
            return Err(error.clone());
        }
        self.state.files.borrow_mut().entry(path.to_path_buf()).or_default().push_str(content);
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), PlatformError> {
        if let Some(error) = self.state.file_errors.borrow().get(path) {
            return Err(error.clone());
        }
        if self.state.files.borrow_mut().remove(path).is_some() {
            Ok(())
        } else {
            Err(PlatformError::new(PlatformErrorKind::NotFound, format!("No such file: {}", path.display())))
        }
    }

    fn path_exists(&self, path: &Path) -> bool {
        self.state.files.borrow().contains_key(path) || self.state.dirs.borrow().contains(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.state.files.borrow().contains_key(path)
    }

    fn emit_output(&self, line: &str) {
        self.state.output.borrow_mut().push(line.to_owned());
    }

    fn emit_warning(&self, line: usize, column: usize, message: &str) {
        self.state.warnings.borrow_mut().push((line, column, message.to_owned()));
    }

    fn read_line(&self) -> Result<String, PlatformError> {
        if let Some(error) = self.state.input_error.borrow().clone() {
            return Err(error);
        }
        self.state.input.borrow_mut().pop_front().ok_or_else(|| {
            PlatformError::new(PlatformErrorKind::Other, "No more input available")
        })
    }

    fn env_vars(&self) -> Vec<(String, String)> {
        self.state.env.borrow().clone()
    }

    fn os(&self) -> &'static str {
        self.state.os.get()
    }

    fn arch(&self) -> &'static str {
        self.state.arch.get()
    }

    fn current_dir(&self) -> Result<PathBuf, PlatformError> {
        self.state.current_dir.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_io_preserves_kind_mapping() {
        assert_eq!(PlatformError::from_io(io::Error::new(io::ErrorKind::NotFound, "gone")).kind, PlatformErrorKind::NotFound);
        assert_eq!(
            PlatformError::from_io(io::Error::new(io::ErrorKind::PermissionDenied, "denied")).kind,
            PlatformErrorKind::PermissionDenied
        );
        for kind in [io::ErrorKind::Other, io::ErrorKind::InvalidInput, io::ErrorKind::AlreadyExists, io::ErrorKind::TimedOut] {
            assert_eq!(PlatformError::from_io(io::Error::new(kind, "x")).kind, PlatformErrorKind::Other, "kind {kind:?}");
        }
    }

    #[test]
    fn from_io_preserves_complete_display_detail() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "gone fishing");
        let expected = io_error.to_string();
        assert_eq!(PlatformError::from_io(io_error).detail, expected);
        let os_error = io::Error::from_raw_os_error(2);
        let error = PlatformError::from_io(os_error);
        assert_eq!(error.kind, PlatformErrorKind::NotFound);
        assert!(error.detail.contains("os error 2"), "detail: {}", error.detail);
        assert_eq!(error.to_string(), error.detail);
    }

    #[test]
    fn native_reports_host_os_arch_and_working_directory() {
        let native = NativePlatform;
        assert_eq!(native.os(), std::env::consts::OS);
        assert_eq!(native.arch(), std::env::consts::ARCH);
        assert_eq!(native.current_dir().unwrap(), std::env::current_dir().unwrap());
    }

    #[test]
    fn native_missing_file_maps_to_not_found() {
        let missing = std::env::temp_dir().join("luca_platform_no_such_file.lucc");
        let _ = std::fs::remove_file(&missing);
        let error = NativePlatform.read_file(&missing).unwrap_err();
        assert_eq!(error.kind, PlatformErrorKind::NotFound);
        assert!(!NativePlatform.path_exists(&missing));
    }

    #[test]
    fn fake_scripts_files_and_failures_deterministically() {
        let fake = FakePlatform::new();
        let path = PathBuf::from("/fake/data.txt");
        fake.add_file(&path, "hello");
        assert!(fake.path_exists(&path));
        assert_eq!(fake.read_file(&path).unwrap(), "hello");

        fake.write_file(&PathBuf::from("/fake/new.txt"), "v1").unwrap();
        assert_eq!(fake.read_file(&PathBuf::from("/fake/new.txt")).unwrap(), "v1");
        fake.append_file(&PathBuf::from("/fake/new.txt"), "+v2").unwrap();
        assert_eq!(fake.read_file(&PathBuf::from("/fake/new.txt")).unwrap(), "v1+v2");
        fake.remove_file(&PathBuf::from("/fake/new.txt")).unwrap();
        assert!(!fake.path_exists(&PathBuf::from("/fake/new.txt")));

        let denied = PlatformError::new(PlatformErrorKind::PermissionDenied, "denied");
        fake.fail_file(&path, denied.clone());
        assert_eq!(fake.read_file(&path).unwrap_err(), denied);
        assert_eq!(fake.write_file(&path, "x").unwrap_err(), denied);
    }

    #[test]
    fn fake_scripts_input_env_output_and_identity() {
        let fake = FakePlatform::new();
        fake.set_input(vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(fake.read_line().unwrap(), "a");
        assert_eq!(fake.read_line().unwrap(), "b");
        let eof = fake.read_line().unwrap_err();
        assert_eq!(eof.detail, "No more input available");

        fake.fail_input(PlatformError::new(PlatformErrorKind::Other, "boom"));
        assert_eq!(fake.read_line().unwrap_err().detail, "boom");
        fake.clear_input_error();
        fake.push_input("c");
        assert_eq!(fake.read_line().unwrap(), "c");

        fake.set_env(vec![("B".to_owned(), "2".to_owned()), ("A".to_owned(), "1".to_owned())]);
        assert_eq!(fake.env_vars(), [("B".to_owned(), "2".to_owned()), ("A".to_owned(), "1".to_owned())]);

        fake.emit_output("out");
        fake.emit_warning(3, 7, "careful");
        assert_eq!(fake.output(), ["out"]);
        assert_eq!(fake.warnings(), [(3, 7, "careful".to_owned())]);

        fake.set_os_arch("windows", "x86_64");
        assert_eq!((fake.os(), fake.arch()), ("windows", "x86_64"));
        fake.set_current_dir(Err(PlatformError::unsupported("no cwd")));
        let failure = fake.current_dir().unwrap_err();
        assert_eq!(failure.kind, PlatformErrorKind::Unsupported);
    }

    #[test]
    fn fake_clones_share_state() {
        let fake = FakePlatform::new();
        let shared = fake.clone();
        fake.add_file("/fake/shared.txt", "v");
        fake.push_input("in");
        shared.emit_output("out");
        shared.emit_warning(1, 2, "warn");
        assert_eq!(fake.output(), ["out"]);
        assert_eq!(shared.output(), ["out"]);
        assert_eq!(fake.warnings(), [(1, 2, "warn".to_owned())]);
        assert_eq!(shared.read_file(&PathBuf::from("/fake/shared.txt")).unwrap(), "v");
        assert_eq!(shared.read_line().unwrap(), "in");
        assert!(fake.read_line().is_err());
    }

    #[test]
    fn fake_distinguishes_files_from_directories() {
        let fake = FakePlatform::new();
        let file = PathBuf::from("/fake/lib/Utils.lucc");
        let dir = PathBuf::from("/fake/lib/DirMod.lucc");
        let missing = PathBuf::from("/fake/lib/Nope.lucc");
        fake.add_file(&file, "x");
        fake.add_dir(&dir);
        assert!(fake.path_exists(&file));
        assert!(fake.is_file(&file));
        assert!(fake.path_exists(&dir));
        assert!(!fake.is_file(&dir));
        assert!(!fake.path_exists(&missing));
        assert!(!fake.is_file(&missing));
    }

    #[test]
    fn fake_instances_are_isolated() {
        let first = FakePlatform::new();
        let second = FakePlatform::new();
        first.add_file("/fake/shared.txt", "first");
        first.push_input("x");
        first.emit_output("first-out");
        first.set_os_arch("linux", "aarch64");

        assert!(!second.path_exists(&PathBuf::from("/fake/shared.txt")));
        assert!(second.read_line().is_err());
        assert!(second.output().is_empty());
        assert_eq!((second.os(), second.arch()), (std::env::consts::OS, std::env::consts::ARCH));
    }
}
