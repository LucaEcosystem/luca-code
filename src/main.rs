use std::{env, fs, path::PathBuf, process};

use luca_code::{diagnostics_to_json, interpreter::Interpreter, RunOutcome, PRODUCT_VERSION};

const USAGE: &str = "Usage: luca <program.lucc> [args...]\n       luca check <program.lucc> [--format human|json]\n\nRun a Luca Code program. Extra arguments are visible to the program via System.args().\n\nOptions:\n  --help     Show this help and exit\n  --version  Show the Luca Code version and exit\n\nCheck options:\n  luca check <program.lucc> [--format human|json]\n    Validate a program without executing it. Prints diagnostics and exits\n    0 when valid, 1 when validation errors are found, 2 for usage or\n    internal validator failures.\n\nExamples:\n  luca hello.lucc\n  luca hello.lucc extra args here\n  luca check hello.lucc\n  luca check hello.lucc --format json";

const USAGE_CHECK: &str = "Usage: luca check <program.lucc> [--format human|json]\n\nValidate a Luca Code program without executing it. Prints diagnostics and exits\n0 when valid, 1 when validation errors are found, 2 for usage or internal\nvalidator failures.\n\nOptions:\n  --format human|json  Output format (default: human)\n  --help               Show this help and exit\n\nExamples:\n  luca check hello.lucc\n  luca check hello.lucc --format json";

fn main() {
    let mut raw = env::args();
    let executable = raw.next().unwrap_or_else(|| "luca".to_owned());
    let path = match raw.next() {
        Some(path) => path,
        None => {
            eprintln!("{USAGE}");
            process::exit(2);
        }
    };

    if path == "--help" || path == "-h" {
        println!("{USAGE}");
        return;
    }
    if path == "--version" || path == "-V" {
        println!("luca {PRODUCT_VERSION}");
        return;
    }
    if path == "check" {
        run_check(raw);
        return;
    }

    if !path.ends_with(".lucc") {
        eprintln!("Luca Code source files must use the .lucc extension");
        process::exit(2);
    }

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Could not read {path}: {error}");
            process::exit(1);
        }
    };

    let mut search_paths: Vec<PathBuf> = Vec::new();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        if !parent.as_os_str().is_empty() {
            search_paths.push(parent.to_path_buf());
        }
    }
    if let Ok(cwd) = env::current_dir() {
        if !search_paths.contains(&cwd) {
            search_paths.push(cwd);
        }
    }

    // `System.args()` observes the full process argument vector.
    let mut cli_args = vec![executable, path.clone()];
    cli_args.extend(raw);

    match Interpreter::with_search_paths(search_paths)
        .with_cli_args(cli_args)
        .run_outcome(&source)
    {
        Ok(RunOutcome::Completed(output)) => {
            for line in &output {
                println!("{line}");
            }
        }
        Ok(RunOutcome::Exited { code, output }) => {
            for line in &output {
                println!("{line}");
            }
            process::exit(code);
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

/// Validate a program without executing it.
///
/// Exit codes: 0 valid, 1 validation errors, 2 usage/internal failure.
/// Validation diagnostics go to stdout (human `Error at L:C` lines or a
/// stable JSON array); internal failures go to stderr with a
/// `luca check:` prefix and never as diagnostics.
fn run_check(args: env::Args) {
    let mut target: Option<String> = None;
    let mut format = "human".to_owned();
    let mut format_seen = false;
    let mut pending_format = false;

    for arg in args {
        if pending_format {
            format = arg;
            format_seen = true;
            pending_format = false;
            continue;
        }
        if arg == "--format" {
            pending_format = true;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--format=") {
            format = value.to_owned();
            format_seen = true;
            continue;
        }
        if (arg == "--help" || arg == "-h") && target.is_none() && !format_seen && !pending_format {
            println!("{USAGE_CHECK}");
            return;
        }
        if target.is_none() {
            target = Some(arg);
        } else {
            eprintln!("{USAGE_CHECK}");
            process::exit(2);
        }
    }

    if pending_format {
        eprintln!("{USAGE_CHECK}");
        process::exit(2);
    }
    if format != "human" && format != "json" {
        eprintln!("{USAGE_CHECK}");
        process::exit(2);
    }
    let target = match target {
        Some(target) => target,
        None => {
            eprintln!("{USAGE_CHECK}");
            process::exit(2);
        }
    };
    if target == "--help" || target == "-h" {
        println!("{USAGE_CHECK}");
        return;
    }
    if !target.ends_with(".lucc") {
        eprintln!("Luca Code source files must use the .lucc extension");
        process::exit(2);
    }

    let entry = PathBuf::from(&target);
    let mut search_paths: Vec<PathBuf> = Vec::new();
    if let Some(parent) = std::path::Path::new(&target).parent() {
        if !parent.as_os_str().is_empty() {
            search_paths.push(parent.to_path_buf());
        }
    }
    if let Ok(cwd) = env::current_dir() {
        if !search_paths.contains(&cwd) {
            search_paths.push(cwd);
        }
    }

    let diagnostics = match luca_code::check_file(&entry, &search_paths) {
        Ok(diagnostics) => diagnostics,
        Err(failure) => {
            eprintln!("{failure}");
            process::exit(2);
        }
    };
    if diagnostics.is_empty() {
        if format == "json" {
            println!("[]");
        }
        return;
    }
    if format == "json" {
        println!("{}", diagnostics_to_json(&diagnostics));
    } else {
        for diagnostic in &diagnostics {
            println!("{diagnostic}");
        }
    }
    process::exit(1);
}
