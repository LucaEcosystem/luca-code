//! Deterministic module-loading tests through `Platform` (M1b-5 B1).
//!
//! Every test scripts an in-memory filesystem on a shared `FakePlatform`
//! and asserts the frozen module diagnostics, source locations, and
//! import behavior. No host filesystem access occurs.
use luca_code::{
    error::LucaError,
    interpreter::{Interpreter, RunOutcome},
    FakePlatform, Platform, PlatformError, PlatformErrorKind,
};
use std::path::PathBuf;

fn interpreter_with(fake: &FakePlatform, search_dirs: &[&str]) -> Interpreter {
    Interpreter::with_search_paths(search_dirs.iter().map(PathBuf::from).collect())
        .with_platform(Box::new(fake.clone()))
}

fn run_module_source(
    fake: &FakePlatform,
    search_dirs: &[&str],
    source: &str,
) -> Result<RunOutcome, LucaError> {
    interpreter_with(fake, search_dirs).run_outcome(source)
}

#[test]
fn imports_resolve_and_run_successfully() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/Utils.lucc", "def var version = \"1.0\"\n");
    let outcome = run_module_source(&fake, &["/fake/lib"], "import Utils\nprint(Utils.version)\n").unwrap();
    assert_eq!(outcome, RunOutcome::Completed(vec!["1.0".to_owned()]));
}

#[test]
fn repeated_imports_initialize_once() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/Utils.lucc", "def var version = \"1.0\"\n");
    let outcome =
        run_module_source(&fake, &["/fake/lib"], "import Utils\nimport Utils\nprint(\"ok\")\n").unwrap();
    assert_eq!(outcome, RunOutcome::Completed(vec!["ok".to_owned()]));
}

#[test]
fn missing_module_reports_location() {
    let fake = FakePlatform::new();
    let error = run_module_source(&fake, &["/fake/lib"], "import Nope\n").unwrap_err();
    assert_eq!(error.to_string(), "Error at 1:1: missing module 'Nope'");
}

#[test]
fn unreadable_module_reports_location_and_detail() {
    let fake = FakePlatform::new();
    // The file exists (resolvable) but cannot be read, mirroring a native
    // permission-denied file whose metadata still reports a regular file.
    fake.add_file("/fake/lib/Broken.lucc", "def var x = 1\n");
    fake.fail_file(
        "/fake/lib/Broken.lucc",
        PlatformError::new(PlatformErrorKind::PermissionDenied, "denied"),
    );
    let error = run_module_source(&fake, &["/fake/lib"], "import Broken\n").unwrap_err();
    assert_eq!(error.to_string(), "Error at 1:1: Could not read module 'Broken': denied");
}

#[test]
fn directory_named_like_module_is_not_a_candidate() {
    let fake = FakePlatform::new();
    fake.add_dir("/fake/lib/DirMod.lucc");
    assert!(fake.path_exists(&PathBuf::from("/fake/lib/DirMod.lucc")));
    assert!(!fake.is_file(&PathBuf::from("/fake/lib/DirMod.lucc")));
    let error = run_module_source(&fake, &["/fake/lib"], "import DirMod\n").unwrap_err();
    assert_eq!(error.to_string(), "Error at 1:1: missing module 'DirMod'");
}

#[test]
fn circular_imports_report_cycle() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/CircA.lucc", "import CircB\ndef var a = 1\n");
    fake.add_file("/fake/lib/CircB.lucc", "import CircA\ndef var b = 1\n");
    let error = run_module_source(&fake, &["/fake/lib"], "import CircA\n").unwrap_err();
    assert!(error.to_string().contains("Circular import 'CircA'"), "got: {error}");
}

#[test]
fn nested_import_inside_function_resolves_on_call() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/Helper.lucc", "def var answer = 42\n");
    let outcome = run_module_source(
        &fake,
        &["/fake/lib"],
        "def func run():\n    import Helper\n    print(Helper.answer)\nrun()\n",
    )
    .unwrap();
    assert_eq!(outcome, RunOutcome::Completed(vec!["42".to_owned()]));
}

#[test]
fn nested_missing_import_reports_location() {
    let fake = FakePlatform::new();
    let error = run_module_source(
        &fake,
        &["/fake/lib"],
        "def func run():\n    import DeepMissing\n    print(\"hi\")\nrun()\n",
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "Error at 2:5: missing module 'DeepMissing'");
}

#[test]
fn search_path_order_prefers_first_directory() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/a/Same.lucc", "def var origin = \"a\"\n");
    fake.add_file("/fake/b/Same.lucc", "def var origin = \"b\"\n");
    let outcome =
        run_module_source(&fake, &["/fake/a", "/fake/b"], "import Same\nprint(Same.origin)\n").unwrap();
    assert_eq!(outcome, RunOutcome::Completed(vec!["a".to_owned()]));
}

#[test]
fn syntax_error_inside_module_keeps_wrapping() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/Bad.lucc", "def var = 1\n");
    let error = run_module_source(&fake, &["/fake/lib"], "import Bad\n").unwrap_err();
    assert!(error.to_string().starts_with("Error at "), "got: {error}");
    assert!(error.message.starts_with("In module 'Bad': "), "got: {error}");
}

#[test]
fn runtime_error_inside_module_keeps_wrapping() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/Boom.lucc", "print(10 / 0)\n");
    let error = run_module_source(&fake, &["/fake/lib"], "import Boom\n").unwrap_err();
    assert_eq!(error.to_string(), "Error at 1:10: In module 'Boom': Division by zero");
}

#[test]
fn builtin_modules_take_precedence_over_files() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/Math.lucc", "def var decoy = 1\n");
    let outcome =
        run_module_source(&fake, &["/fake/lib"], "import Math\nprint(Math.abs(0 - 3))\n").unwrap();
    assert_eq!(outcome, RunOutcome::Completed(vec!["3".to_owned()]));
}
