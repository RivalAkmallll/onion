use clap::{Parser as ClapParser, Subcommand};
use onion_compiler::error::OnionError;
use onion_compiler::{ast, checker, codegen, interpreter, lexer, linker, parser};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(ClapParser)]
#[command(
    name = "onion",
    version,
    about = "🧅 Onion — Layered Safety, Blazingly Fast, Developer-Friendly"
)]
struct Cli {    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile to a temp .exe and run it (use --interp for the interpreter)
    Run {
        /// Path to the .oni file
        file: PathBuf,
        /// Use the slow interpreter instead of native code
        #[arg(long)]
        interp: bool,
    },
    /// Compile a .oni file to a native .exe
    Build {
        /// Path to the .oni file
        file: PathBuf,
        /// Output path (default: same name with .exe)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Type-check a .oni file without running it
    Check {
        /// Path to the .oni file
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, interp } => run_file(&file, interp),
        Commands::Build { file, output } => build_file(&file, output),
        Commands::Check { file } => check_file(&file),
    }
}

/// Reads a `.oni` file, validating the extension and stripping any UTF-8 BOM.
fn load_source(path: &Path) -> Option<String> {
    let filename = path.display().to_string();

    if path.extension().and_then(|e| e.to_str()) != Some("oni") {
        eprintln!("error: file must have the `.oni` extension (given: {filename})");
        return None;
    }

    match std::fs::read_to_string(path) {
        Ok(source) => Some(source.trim_start_matches('\u{feff}').to_string()),
        Err(e) => {
            eprintln!("error: failed to read `{filename}`: {e}");
            None
        }
    }
}

fn print_errors(errors: &[OnionError], source: &str, filename: &str) {
    for error in errors {
        eprintln!("{}\n", error.render(source, filename));
    }
    let n = errors.len();
    eprintln!(
        "error: aborting due to {n} previous error{}",
        if n == 1 { "" } else { "s" }
    );
}

/// Pipeline: source -> tokens -> AST -> type check.
fn compile(source: &str) -> Result<ast::Program, Vec<OnionError>> {
    let tokens = lexer::lex(source).map_err(|e| vec![e])?;
    let program = parser::parse(tokens, source.len()).map_err(|e| vec![e])?;
    checker::check(&program)?;
    Ok(program)
}

/// AST -> object file -> linked .exe
fn build_exe(program: &ast::Program, exe: &Path) -> Result<(), OnionError> {
    let object = codegen::compile(program)?;

    let obj_path = exe.with_extension("obj");
    std::fs::write(&obj_path, object).map_err(|e| {
        OnionError::without_span(format!("failed to write `{}`: {e}", obj_path.display()))
    })?;

    let result = linker::link(&obj_path, exe);
    let _ = std::fs::remove_file(&obj_path);
    result.map_err(OnionError::without_span)
}

fn run_file(path: &Path, interp: bool) -> ExitCode {
    let filename = path.display().to_string();
    let Some(source) = load_source(path) else {
        return ExitCode::FAILURE;
    };

    let program = match compile(&source) {
        Ok(program) => program,
        Err(errors) => {
            print_errors(&errors, &source, &filename);
            return ExitCode::FAILURE;
        }
    };

    if interp {
        return match interpreter::run(&program) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                print_errors(&[error], &source, &filename);
                ExitCode::FAILURE
            }
        };
    }

    // Native: build into the temp folder, run, then delete.
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("program");
    let exe = std::env::temp_dir().join(format!("onion-{stem}-{}.exe", std::process::id()));

    if let Err(error) = build_exe(&program, &exe) {
        print_errors(&[error], &source, &filename);
        return ExitCode::FAILURE;
    }

    let status = Command::new(&exe).status();
    let _ = std::fs::remove_file(&exe);

    match status {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(e) => {
            eprintln!("error: could not start the built program: {e}");
            ExitCode::FAILURE
        }
    }
}

fn build_file(path: &Path, output: Option<PathBuf>) -> ExitCode {
    let filename = path.display().to_string();
    let Some(source) = load_source(path) else {
        return ExitCode::FAILURE;
    };

    let program = match compile(&source) {
        Ok(program) => program,
        Err(errors) => {
            print_errors(&errors, &source, &filename);
            return ExitCode::FAILURE;
        }
    };

    let exe = output.unwrap_or_else(|| path.with_extension("exe"));

    match build_exe(&program, &exe) {
        Ok(()) => {
            println!("built: {}", exe.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            print_errors(&[error], &source, &filename);
            ExitCode::FAILURE
        }
    }
}

fn check_file(path: &Path) -> ExitCode {
    let filename = path.display().to_string();
    let Some(source) = load_source(path) else {
        return ExitCode::FAILURE;
    };

    match compile(&source) {
        Ok(_) => {
            println!("ok: no errors found in {filename}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            print_errors(&errors, &source, &filename);
            ExitCode::FAILURE
        }
    }
}