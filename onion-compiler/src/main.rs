use clap::{Parser as ClapParser, Subcommand};
use onion_compiler::error::OnionError;
use onion_compiler::{ast, checker, codegen, interpreter, lexer, linker, parser};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

// Prevent extremely large inputs from exhausting memory
// during the initial source loading stage.
const MAX_SOURCE_SIZE: u64 = 4 * 1024 * 1024;

#[derive(ClapParser)]
#[command(
    name = "onion",
    version,
    about = "Onion - A Rust-inspired programming language",
    long_about = "The Onion Programming Language\n\
                  Compiler, interpreter, and developer tooling.\n\
                  Source extensions: .oni and .onion."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile and run an Onion program
    Run {
        /// Onion source file (.oni or .onion)
        file: PathBuf,

        /// Run with the interpreter
        #[arg(long)]
        interp: bool,
    },

    /// Compile Onion source into a native executable
    Build {
        /// Onion source file (.oni or .onion)
        file: PathBuf,

        /// Output executable
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Check source code without executing it
    Check {
        /// Onion source file (.oni or .onion)
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

// -----------------------------------------------
// SOURCE LOADING
// -----------------------------------------------

fn load_source(path: &Path) -> Option<String> {
    let filename = path.display().to_string();

    let extension = path.extension().and_then(|extension| extension.to_str());

    if !matches!(extension, Some("oni") | Some("onion")) {
        eprintln!("error: expected a .oni or .onion source file: {filename}");
        return None;
    }

    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => {
            eprintln!("error: cannot access `{filename}`: {error}");
            return None;
        }
    };

    if !metadata.is_file() {
        eprintln!("error: input must be a regular file: {filename}");
        return None;
    }

    if metadata.len() > MAX_SOURCE_SIZE {
        eprintln!(
            "error: source file exceeds the {} MiB limit",
            MAX_SOURCE_SIZE / 1024 / 1024
        );
        return None;
    }

    // Read at most MAX_SOURCE_SIZE + 1 bytes so files
    // that grow between metadata and read are rejected.
    use std::io::Read;

    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("error: cannot open `{filename}`: {error}");
            return None;
        }
    };

    let mut bytes = Vec::new();

    if let Err(error) = file.take(MAX_SOURCE_SIZE + 1).read_to_end(&mut bytes) {
        eprintln!("error: cannot read `{filename}`: {error}");
        return None;
    }

    if bytes.len() as u64 > MAX_SOURCE_SIZE {
        eprintln!("error: source exceeds the input size limit");
        return None;
    }

    // Strip UTF-8 BOM if present.
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);

    match String::from_utf8(bytes.to_vec()) {
        Ok(source) => Some(source),

        Err(error) => {
            eprintln!("error: source is not valid UTF-8: {error}");
            None
        }
    }
}

// -----------------------------------------------
// DIAGNOSTICS
// -----------------------------------------------

fn print_errors(errors: &[OnionError], source: &str, filename: &str) {
    for error in errors {
        eprintln!("{}\n", error.render(source, filename));
    }

    let count = errors.len();

    eprintln!(
        "error: aborting due to {count} previous error{}",
        if count == 1 { "" } else { "s" }
    );
}

// -----------------------------------------------
// COMPILER FRONTEND
// -----------------------------------------------

fn compile(source: &str) -> Result<ast::Program, Vec<OnionError>> {
    let tokens = lexer::lex(source).map_err(|error| vec![error])?;

    let program = parser::parse(tokens, source.len()).map_err(|error| vec![error])?;

    checker::check(&program)?;

    Ok(program)
}

// -----------------------------------------------
// NATIVE CODE GENERATION
// -----------------------------------------------

fn build_exe(program: &ast::Program, exe: &Path) -> Result<(), OnionError> {
    let object = codegen::compile(program)?;

    let obj_path = exe.with_extension("obj");

    fs::write(&obj_path, object).map_err(|error| {
        OnionError::without_span(format!("failed to write `{}`: {error}", obj_path.display()))
    })?;

    let result = linker::link(&obj_path, exe);

    let _ = fs::remove_file(&obj_path);

    result.map_err(OnionError::without_span)
}

// -----------------------------------------------
// RUN COMMAND
// -----------------------------------------------

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

    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("program");

    let exe = std::env::temp_dir().join(format!("onion-{stem}-{}.exe", std::process::id()));

    if let Err(error) = build_exe(&program, &exe) {
        print_errors(&[error], &source, &filename);
        return ExitCode::FAILURE;
    }

    let status = Command::new(&exe).status();

    let _ = fs::remove_file(&exe);

    match status {
        Ok(status) => {
            let code = status
                .code()
                .and_then(|value| u8::try_from(value).ok())
                .unwrap_or(1);

            ExitCode::from(code)
        }

        Err(error) => {
            eprintln!("error: could not execute the program: {error}");

            ExitCode::FAILURE
        }
    }
}

// -----------------------------------------------
// BUILD COMMAND
// -----------------------------------------------

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

    // Do not overwrite the input source file.
    if exe == path {
        eprintln!("error: output path must differ from the source file");
        return ExitCode::FAILURE;
    }

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

// -----------------------------------------------
// CHECK COMMAND
// -----------------------------------------------

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
#[cfg(test)]
mod security_tests {
    use super::*;
    use std::io::Write;

    fn temporary_source(name: &str, content: &[u8]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "onion-security-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));

        fs::create_dir_all(&dir).unwrap();

        let path = dir.join(name);
        let mut file = fs::File::create(&path).unwrap();

        file.write_all(content).unwrap();

        path
    }

    #[test]
    fn accepts_onion_extension() {
        let path = temporary_source("valid.onion", b"fn main() {}");

        assert!(load_source(&path).is_some());

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn accepts_oni_extension() {
        let path = temporary_source("valid.oni", b"fn main() {}");

        assert!(load_source(&path).is_some());

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_unexpected_extension() {
        let path = temporary_source("program.txt", b"fn main() {}");

        assert!(load_source(&path).is_none());

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn strips_utf8_bom() {
        let path = temporary_source("bom.onion", b"\xEF\xBB\xBFfn main() {}");

        let source = load_source(&path).unwrap();

        assert_eq!(source, "fn main() {}");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_invalid_utf8() {
        let path = temporary_source("invalid.onion", &[0xFF, 0xFE, 0xFD]);

        assert!(load_source(&path).is_none());

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_oversized_source() {
        let path = temporary_source("huge.onion", b"");

        let file = fs::OpenOptions::new().write(true).open(&path).unwrap();

        file.set_len(MAX_SOURCE_SIZE + 1).unwrap();

        assert!(load_source(&path).is_none());

        fs::remove_file(path).unwrap();
    }
}
