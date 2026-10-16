//! `luca check` validation tests: read-only, never executes application code.
use luca_code::{check_file, check_source, diagnostics_to_json};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures")
}

fn unique_temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "luca_check_{}_{}_{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_temp(dir: &std::path::Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn accepts_valid_source_without_executing() {
    let diagnostics = check_source("def var count (int) = 2\nprint(count)\n");
    assert!(diagnostics.is_empty(), "expected no diagnostics, got {diagnostics:?}");
}

#[test]
fn reports_syntax_errors_with_location() {
    let diagnostics = check_source("def var = 1\n");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert!(!diagnostic.message.is_empty());
    assert!(diagnostic.line >= 1);
    assert!(diagnostic.column >= 1);
    assert!(diagnostic.to_string().starts_with("Error at "));
    assert!(diagnostic.to_string().contains(&diagnostic.message));
}

#[test]
fn reports_lexer_errors_with_location() {
    let diagnostics = check_source("def var x = \"unterminated\n");
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].to_string().starts_with("Error at "));
}

#[test]
fn passes_side_effecting_source_through_without_any_effect() {
    let dir = unique_temp_dir("passthrough");
    let sentinel = dir.join("must_not_be_created.txt");
    let sentinel_display = sentinel.display().to_string().replace('\\', "\\\\");
    let source = format!(
        "import File\nimport System\ndef var name = ask \"name\"\nprint(name)\nFile.write(\"{sentinel_display}\", \"data\")\nSystem.exit(0)\n"
    );
    // Validation only: syntactically valid, so no diagnostics ...
    let diagnostics = check_source(&source);
    assert!(diagnostics.is_empty(), "expected pass-through, got {diagnostics:?}");
    // ... and crucially nothing was executed: no prompt, no output file,
    // no process termination (this test still running proves the last one).
    assert!(!sentinel.exists(), "check executed File.write");
}

#[test]
fn check_file_passes_side_effecting_file_without_any_effect() {
    let dir = unique_temp_dir("file_passthrough");
    let sentinel = dir.join("must_not_exist.txt");
    let sentinel_display = sentinel.display().to_string().replace('\\', "\\\\");
    let entry = write_temp(
        &dir,
        "main.lucc",
        &format!(
            "import File\nimport System\ndef var name = ask \"name\"\nprint(name)\nFile.write(\"{sentinel_display}\", \"data\")\nSystem.exit(0)\n"
        ),
    );
    let diagnostics = check_file(&entry, std::slice::from_ref(&dir)).unwrap();
    assert!(diagnostics.is_empty(), "expected pass-through, got {diagnostics:?}");
    assert!(!sentinel.exists(), "check_file executed File.write");
}

#[test]
fn check_file_accepts_valid_program_and_builtin_imports() {
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples").join("hello.lucc");
    let parent = entry.parent().unwrap().to_path_buf();
    let diagnostics = check_file(&entry, &[parent]).unwrap();
    assert!(diagnostics.is_empty(), "expected no diagnostics, got {diagnostics:?}");

    let dir = unique_temp_dir("builtin");
    let entry = write_temp(&dir, "main.lucc", "import Math\nimport Text\nprint(Math.abs(0 - 3))\n");
    let diagnostics = check_file(&entry, &[dir]).unwrap();
    assert!(diagnostics.is_empty(), "builtin imports need no files, got {diagnostics:?}");
}

#[test]
fn check_file_resolves_a_valid_module_import() {
    let dir = unique_temp_dir("valid_import");
    std::fs::copy(fixtures_dir().join("Utils.lucc"), dir.join("Utils.lucc")).unwrap();
    let entry = write_temp(&dir, "main.lucc", "import Utils\nprint(Utils.version)\n");
    let diagnostics = check_file(&entry, &[dir]).unwrap();
    assert!(diagnostics.is_empty(), "expected no diagnostics, got {diagnostics:?}");
}

#[test]
fn check_file_reports_missing_imports_without_executing() {
    let dir = unique_temp_dir("missing");
    let entry = write_temp(&dir, "main.lucc", "import NoSuchModuleXYZ\nprint(\"hi\")\n");
    let diagnostics = check_file(&entry, &[dir]).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "missing module 'NoSuchModuleXYZ'");
    assert_eq!((diagnostics[0].line, diagnostics[0].column), (1, 1));
}

#[test]
fn check_file_resolves_nested_imports() {
    let dir = unique_temp_dir("nested");
    let entry = write_temp(
        &dir,
        "main.lucc",
        "def func run() :\n    import MissingNested\n    print(\"hi\")\n",
    );
    let diagnostics = check_file(&entry, &[dir]).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "missing module 'MissingNested'");
}

#[test]
fn check_file_reports_circular_imports() {
    let dir = unique_temp_dir("circular");
    write_temp(&dir, "CircA.lucc", "import CircB\ndef var a = 1\n");
    write_temp(&dir, "CircB.lucc", "import CircA\ndef var b = 1\n");
    let entry = write_temp(&dir, "main.lucc", "import CircA\nprint(\"hi\")\n");
    let diagnostics = check_file(&entry, &[dir]).unwrap();
    assert!(
        diagnostics.iter().any(|d| d.message.contains("Circular import")),
        "expected circular import diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn check_file_wraps_syntax_errors_inside_imports() {
    let dir = unique_temp_dir("bad_import");
    write_temp(&dir, "Broken.lucc", "def var = 1\n");
    let entry = write_temp(&dir, "main.lucc", "import Broken\nprint(\"hi\")\n");
    let diagnostics = check_file(&entry, &[dir]).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0].message.starts_with("In module 'Broken': "),
        "unexpected message: {}",
        diagnostics[0].message
    );
}

#[test]
fn check_file_distinguishes_internal_failures_from_validation_errors() {
    let missing = std::env::temp_dir().join(format!(
        "luca_check_no_such_entry_{}.lucc",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&missing);
    let failure = check_file(&missing, &[]).unwrap_err();
    assert!(failure.to_string().starts_with("luca check: "), "unexpected: {failure}");
    assert!(!failure.to_string().starts_with("Error at "));
}

#[test]
fn json_output_is_stable_and_escaped() {
    assert_eq!(diagnostics_to_json(&[]), "[]");
    let diagnostics = check_source("def var = 1\n");
    assert_eq!(diagnostics.len(), 1);
    let json = diagnostics_to_json(&diagnostics);
    let expected = format!(
        "[{{\"message\":\"{}\",\"line\":{},\"column\":{}}}]",
        diagnostics[0].message.replace('\\', "\\\\").replace('"', "\\\""),
        diagnostics[0].line,
        diagnostics[0].column
    );
    assert_eq!(json, expected);

    let quoted = vec![luca_code::CheckDiagnostic::new("say \"hi\"\nbye", 2, 3)];
    assert_eq!(
        diagnostics_to_json(&quoted),
        "[{\"message\":\"say \\\"hi\\\"\\nbye\",\"line\":2,\"column\":3}]"
    );
}
