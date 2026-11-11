
use clap::{Parser as ClapParser, Subcommand};
use onion_compiler::error::OnionError;
use onion_compiler::{
    ast, checker, codegen, interpreter, lexer, linker, parser,
};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(ClapParser)]
#[command(
    name = "onion",
    version,
    about = "Onion - A Rust-inspired programming language"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile and run an Onion program
    Run {
        /// Path to a .onion or .oni source file
        file: PathBuf,

        /// Use the interpreter instead of native compilation
        #[arg(long)]
        interp: bool,
    },

    /// Compile an Onion program into a native executable
    Build {
        /// Path to a .onion or .oni source file
        file: PathBuf,

        /// Output executable path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Check an Onion program without executing it
    Check {
        /// Path to a .onion or .oni source file
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

/// Reads an Onion source file.
///
/// Supported extensions:
/// - .onion
/// - .oni
///
/// UTF-8 BOM is removed automatically.
fn load_source(path: &Path) -> Option<String> {
    let filename = path.display().to_string();

    let extension = path
        .extension()
        .and_then(|extension| extension.to_str());

    if !matches!(extension, Some("onion") | Some("oni")) {
        eprintln!(
            "error: expected an Onion source file (.onion or .oni), got `{filename}`"
        );
        return None;
    }

    match std::fs::read_to_string(path) {
        Ok(source) => {
            let source = source.trim_start_matches('\u{feff}');
            Some(source.to_string())
        }

        Err(error) => {
            eprintln!(
                "error: failed to read `{filename}`: {error}"
            );
            None
        }
    }
}

/// Print compiler diagnostics.
fn print_errors(
    errors: &[OnionError],
    source: &str,
    filename: &str,
) {
    for error in errors {
        eprintln!(
            "{}\n",
            error.render(source, filename)
        );
    }

    let count = errors.len();

    eprintln!(
        "error: aborting due to {count} previous error{}",
        if count == 1 { "" } else { "s" }
    );
}

/// Onion compilation pipeline:
///
/// Source code
///     ↓
/// Lexer
///     ↓
/// Parser
///     ↓
/// Abstract Syntax Tree
///     ↓
/// Type Checker
///     ↓
/// Validated Program
fn compile(
    source: &str,
) -> Result<ast::Program, Vec<OnionError>> {
    let tokens = lexer::lex(source)
        .map_err(|error| vec![error])?;

    let program = parser::parse(tokens, source.len())
        .map_err(|error| vec![error])?;

    checker::check(&program)?;

    Ok(program)
}

/// Compile the AST into a native executable.
fn build_exe(
    program: &ast::Program,
    exe: &Path,
) -> Result<(), OnionError> {
    let object = codegen::compile(program)?;

    let obj_path = exe.with_extension("obj");

    std::fs::write(&obj_path, object).map_err(|error| {
        OnionError::without_span(format!(
            "failed to write `{}`: {error}",
            obj_path.display()
        ))
    })?;

    let result = linker::link(&obj_path, exe);

    // Remove the temporary object file.
    let _ = std::fs::remove_file(&obj_path);

    result.map_err(OnionError::without_span)
}

/// Run an Onion program.
///
/// Default: native compilation
/// Optional: interpreter mode
fn run_file(
    path: &Path,
    interp: bool,
) -> ExitCode {
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

    // Interpreter mode
    if interp {
        return match interpreter::run(&program) {
            Ok(()) => ExitCode::SUCCESS,

            Err(error) => {
                print_errors(&[error], &source, &filename);
                ExitCode::FAILURE
            }
        };
    }

    // Native compilation mode
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("program");

    let exe = std::env::temp_dir().join(format!(
        "onion-{stem}-{}.exe",
        std::process::id()
    ));

    if let Err(error) = build_exe(&program, &exe) {
        print_errors(&[error], &source, &filename);
        return ExitCode::FAILURE;
    }

    // Execute compiled program
    let status = Command::new(&exe).status();

    // Clean up temporary executable
    let _ = std::fs::remove_file(&exe);

    match status {
        Ok(status) => {
            ExitCode::from(
                status.code().unwrap_or(1) as u8
            )
        }

        Err(error) => {
            eprintln!(
                "error: could not start the built program: {error}"
            );

            ExitCode::FAILURE
        }
    }
}

/// Build an Onion program into an executable.
fn build_file(
    path: &Path,
    output: Option<PathBuf>,
) -> ExitCode {
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

    let exe = output.unwrap_or_else(|| {
        path.with_extension("exe")
    });

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

/// Validate Onion source code without executing it.
fn check_file(path: &Path) -> ExitCode {
    let filename = path.display().to_string();

    let Some(source) = load_source(path) else {
        return ExitCode::FAILURE;
    };

    match compile(&source) {
        Ok(_) => {
            println!(
                "ok: no errors found in {filename}"
            );

            ExitCode::SUCCESS
        }

        Err(errors) => {
            print_errors(&errors, &source, &filename);
            ExitCode::FAILURE
        }
    }
}
