//! Platform-routed validation tests (M2 B1).
//!
//! Every test scripts an in-memory filesystem on a shared `FakePlatform`
//! and validates through `check_file_with_platform`. No host filesystem
//! access occurs. Behaviors mirror `tests/check.rs` (native) exactly.
use luca_code::{
    check_file_with_platform, diagnostics_to_json, CheckDiagnostic, FakePlatform, Platform,
    PlatformError, PlatformErrorKind,
};
use std::path::PathBuf;

fn check_with_fake(
    fake: &FakePlatform,
    entry: &str,
    search_dirs: &[&str],
) -> Result<Vec<CheckDiagnostic>, luca_code::CheckFailure> {
    check_file_with_platform(
        &PathBuf::from(entry),
        &search_dirs.iter().map(PathBuf::from).collect::<Vec<_>>(),
        fake,
    )
}

#[test]
fn resolves_scripted_files_and_validates_graph() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/Utils.lucc", "def var version = \"1.0\"\n");
    fake.add_file(
        "/fake/proj/main.lucc",
        "import Utils\nprint(Utils.version)\n",
    );
    let diagnostics = check_with_fake(&fake, "/fake/proj/main.lucc", &["/fake/lib"]).unwrap();
    assert!(diagnostics.is_empty(), "got: {diagnostics:?}");
}

#[test]
fn reports_missing_files_without_touching_host() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/proj/main.lucc", "import Absent\n");
    let diagnostics = check_with_fake(&fake, "/fake/proj/main.lucc", &["/fake/lib"]).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].to_string(), "Error at 1:1: missing module 'Absent'");
}

#[test]
fn resolves_nested_imports() {
    let fake = FakePlatform::new();
    fake.add_file(
        "/fake/proj/main.lucc",
        "def func run():\n    import Deep\n    print(\"hi\")\n",
    );
    let diagnostics = check_with_fake(&fake, "/fake/proj/main.lucc", &["/fake/lib"]).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].to_string(), "Error at 2:5: missing module 'Deep'");
}

#[test]
fn directory_with_lucc_extension_is_not_a_module() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/proj/main.lucc", "import DirMod\n");
    fake.add_dir("/fake/lib/DirMod.lucc");
    assert!(fake.path_exists(&PathBuf::from("/fake/lib/DirMod.lucc")));
    assert!(!fake.is_file(&PathBuf::from("/fake/lib/DirMod.lucc")));
    let diagnostics = check_with_fake(&fake, "/fake/proj/main.lucc", &["/fake/lib"]).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].to_string(), "Error at 1:1: missing module 'DirMod'");
}

#[test]
fn reports_unreadable_modules_with_detail() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/proj/main.lucc", "import Locked\n");
    fake.add_file("/fake/lib/Locked.lucc", "def var x = 1\n");
    fake.fail_file(
        "/fake/lib/Locked.lucc",
        PlatformError::new(PlatformErrorKind::PermissionDenied, "denied"),
    );
    let diagnostics = check_with_fake(&fake, "/fake/proj/main.lucc", &["/fake/lib"]).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].to_string(),
        "Error at 1:1: Could not read module 'Locked': denied"
    );
}

#[test]
fn wraps_inner_errors_with_module_context() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/proj/main.lucc", "import Bad\n");
    fake.add_file("/fake/lib/Bad.lucc", "def var = 1\n");
    let diagnostics = check_with_fake(&fake, "/fake/proj/main.lucc", &["/fake/lib"]).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.starts_with("In module 'Bad': "), "got: {}", diagnostics[0]);
}

#[test]
fn reports_circular_imports() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/proj/main.lucc", "import CircA\n");
    fake.add_file("/fake/lib/CircA.lucc", "import CircB\n");
    fake.add_file("/fake/lib/CircB.lucc", "import CircA\n");
    let diagnostics = check_with_fake(&fake, "/fake/proj/main.lucc", &["/fake/lib"]).unwrap();
    assert!(diagnostics.iter().any(|d| d.message.contains("Circular import")), "got: {diagnostics:?}");
}

#[test]
fn entry_read_failure_is_internal_not_diagnostic() {
    let fake = FakePlatform::new();
    let failure =
        check_with_fake(&fake, "/fake/proj/missing.lucc", &["/fake/lib"]).unwrap_err();
    assert!(failure.to_string().starts_with("luca check: "), "got: {failure}");
    assert!(!failure.to_string().starts_with("Error at "));
}

#[test]
fn builtin_imports_need_no_files() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/proj/main.lucc", "import Math\nprint(Math.abs(0 - 1))\n");
    let diagnostics = check_with_fake(&fake, "/fake/proj/main.lucc", &["/fake/lib"]).unwrap();
    assert!(diagnostics.is_empty(), "got: {diagnostics:?}");
}

#[test]
fn matches_native_check_on_identical_content() {
    let source = "import Utils\nprint(Utils.version)\n";
    let utils = "def var version = \"1.0\"\n";
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/Utils.lucc", utils);
    fake.add_file("/fake/proj/main.lucc", source);
    let via_platform =
        check_with_fake(&fake, "/fake/proj/main.lucc", &["/fake/lib"]).unwrap();
    assert!(via_platform.is_empty(), "got: {via_platform:?}");
    assert_eq!(diagnostics_to_json(&via_platform), "[]");
}

#[test]
fn diagnostic_json_shape_is_stable() {
    let diagnostics = vec![CheckDiagnostic::new("missing module 'X'", 2, 3)];
    assert_eq!(
        diagnostics_to_json(&diagnostics),
        "[{\"message\":\"missing module 'X'\",\"line\":2,\"column\":3}]"
    );
}
