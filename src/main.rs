use std::{env, fs, path::PathBuf, process};

use luca_code::interpreter::Interpreter;

fn main() {
    let mut raw = env::args();
    let executable = raw.next().unwrap_or_else(|| "luca".to_owned());
    let path = match raw.next() {
        Some(path) => path,
        None => {
            eprintln!("Usage: luca <program.lucc> [args...]");
            process::exit(2);
        }
    };

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

    if let Err(error) = Interpreter::with_search_paths(search_paths)
        .with_cli_args(cli_args)
        .run(&source)
    {
        eprintln!("{error}");
        process::exit(1);
    }
}
