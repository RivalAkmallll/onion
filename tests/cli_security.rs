
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

const MAX_SOURCE_SIZE: u64 = 4 * 1024 * 1024;

struct TestFile {
    directory: PathBuf,
    path: PathBuf,
}

impl TestFile {
    fn new(name: &str, contents: &[u8]) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        let directory = std::env::temp_dir().join(format!(
            "onion-cli-security-{}-{id}",
            std::process::id()
        ));

        fs::create_dir_all(&directory).unwrap();

        let path = directory.join(name);
        fs::write(&path, contents).unwrap();

        Self { directory, path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn run_check(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_onion"))
        .arg("check")
        .arg(path)
        .output()
        .expect("failed to start Onion compiler")
}

fn diagnostic(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn accepts_oni_files() {
    let source = TestFile::new(
        "main.oni",
        b"fn main() { println!(\"Hello\"); }",
    );

    let output = run_check(source.path());

    assert!(
        output.status.success(),
        "{}",
        diagnostic(&output)
    );
}

#[test]
fn accepts_onion_files() {
    let source = TestFile::new(
        "main.onion",
        b"fn main() { println!(\"Hello\"); }",
    );

    let output = run_check(source.path());

    assert!(
        output.status.success(),
        "{}",
        diagnostic(&output)
    );
}

#[test]
fn rejects_other_extensions() {
    let source = TestFile::new(
        "main.txt",
        b"fn main() {}",
    );

    let output = run_check(source.path());

    assert!(!output.status.success());
    assert!(
        diagnostic(&output).contains(".onion")
            || diagnostic(&output).contains(".oni")
    );
}

#[test]
fn accepts_utf8_bom() {
    let source = TestFile::new(
        "bom.onion",
        b"\xEF\xBB\xBFfn main() {}",
    );

    let output = run_check(source.path());

    assert!(
        output.status.success(),
        "{}",
        diagnostic(&output)
    );
}

#[test]
fn rejects_invalid_utf8() {
    let source = TestFile::new(
        "invalid.onion",
        &[0xFF, 0xFE, 0xFD],
    );

    let output = run_check(source.path());

    assert!(!output.status.success());
}

#[test]
fn rejects_oversized_source() {
    let source = TestFile::new(
        "oversized.onion",
        b"",
    );

    let file = fs::OpenOptions::new()
        .write(true)
        .open(source.path())
        .unwrap();

    file.set_len(MAX_SOURCE_SIZE + 1).unwrap();

    let output = run_check(source.path());

    assert!(!output.status.success());
    assert!(
        diagnostic(&output).contains("limit")
            || diagnostic(&output).contains("exceeds")
    );
}

#[test]
fn rejects_nonexistent_file() {
    let source = TestFile::new(
        "existing.onion",
        b"fn main() {}",
    );

    let missing = source.directory.join("missing.onion");
    let output = run_check(&missing);

    assert!(!output.status.success());
}
