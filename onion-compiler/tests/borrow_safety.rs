use onion_compiler::{checker, lexer, parser};

fn verify(body: &str) -> Result<(), Vec<onion_compiler::error::OnionError>> {
    let source = format!("fn main() {{ {body} }}");
    let program = parser::parse(lexer::lex(&source).unwrap(), source.len()).unwrap();
    checker::check(&program)
}

#[test]
fn direct_read_during_exclusive_borrow_is_rejected() {
    let errors = verify(r#"let mut x = 1 let r = &mut x println!("{}", x) *r = 2"#).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("cannot read `x` while mutably borrowed")));
}

#[test]
fn final_reference_use_releases_straight_line_borrow() {
    assert!(verify(r#"let mut x = 1 let r = &mut x *r = 2 println!("{}", x)"#).is_ok());
    assert!(verify(r#"let mut x = 1 let r = &mut x println!("{}", *r) x = 3"#).is_ok());
}

#[test]
fn later_reference_use_keeps_exclusive_borrow_live() {
    assert!(
        verify(r#"let mut x = 1 let r = &mut x *r = 2 println!("{}", x) println!("{}", *r)"#)
            .is_err()
    );
    assert!(verify(r#"let mut x = 1 let r = &mut x *r = x"#).is_err());
}

#[test]
fn control_flow_does_not_end_borrows_early() {
    for body in [
        r#"let mut x = 1 let r = &mut x if true { println!("{}", x) } *r = 2"#,
        r#"let mut x = 1 let r = &mut x *r = 2 if true { println!("{}", x) }"#,
        r#"let mut x = 1 let r = &mut x loop { *r = 2 println!("{}", x) break }"#,
        r#"let mut x = 1 loop { let r = &mut x *r = 2 println!("{}", x) continue }"#,
    ] {
        assert!(verify(body).is_err(), "accepted: {body}");
    }
}

#[test]
fn shadowing_cannot_redirect_native_reference_writes() {
    assert!(verify(r#"let mut x = 1 let r = &mut x if true { let mut x = 9 *r = 2 }"#).is_err());
    assert!(verify(r#"let mut x = 1 let r = &mut x let x = *r println!("{}", x)"#).is_err());
}

#[test]
fn loop_exits_cannot_restore_moved_ownership() {
    for body in [
        r#"let mut s = "a" loop { let t = s break s = "b" } println!("{}", s)"#,
        r#"let mut s = "a" loop { let t = s continue s = "b" }"#,
        r#"let mut s = "a" let t = s loop { break s = "b" } println!("{}", s)"#,
    ] {
        assert!(verify(body).is_err(), "accepted: {body}");
    }
}

#[test]
fn loop_reference_reassignment_is_conservatively_rejected() {
    assert!(verify(
        r#"let a = 1 let b = 2 let mut r = &a loop { r = &b break } println!("{}", *r)"#
    )
    .is_err());
}

#[test]
fn local_loop_ownership_and_scoped_borrows_remain_valid() {
    assert!(verify(r#"loop { let s = "a" let t = s println!("{}", t) break }"#).is_ok());
    assert!(verify(r#"let mut x = 1 if true { let r = &mut x *r = 2 } println!("{}", x)"#).is_ok());
}

#[test]
fn branch_join_keeps_outer_borrow_and_move_restrictions() {
    for body in [
        r#"let mut x = 1 let r = &mut x if true { *r = 2 } else { *r = 3 } println!("{}", x)"#,
        r#"let mut s = "a" if true { let t = s } println!("{}", s)"#,
        r#"let a = 1 let b = 2 let mut r = &a if true { r = &b } println!("{}", *r)"#,
    ] {
        assert!(verify(body).is_err(), "accepted: {body}");
    }
}

#[test]
fn shared_references_keep_lexical_lifetimes() {
    assert!(verify(r#"let mut x = 1 let r = &x println!("{}", *r) x = 2"#).is_err());
}

#[test]
fn borrow_creation_reports_conflicts_without_treating_it_as_a_read() {
    for (body, message) in [
        (
            "let mut x = 1 let r = &mut x let s = &mut x",
            "already borrowed",
        ),
        (
            "let mut x = 1 let r = &mut x let s = &x",
            "immutably borrow",
        ),
    ] {
        let errors = verify(body).unwrap_err();
        assert!(errors.iter().any(|error| error.message.contains(message)));
        assert!(!errors
            .iter()
            .any(|error| error.message.contains("cannot read")));
    }
}

#[cfg(windows)]
#[test]
fn interpreter_and_native_agree_on_supported_borrows() {
    use std::fs;
    use std::process::{Command, Output, Stdio};
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    fn output(command: &mut Command) -> Output {
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if child.try_wait().unwrap().is_some() {
                return child.wait_with_output().unwrap();
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                let output = child.wait_with_output().unwrap();
                panic!("test process exceeded 20 seconds: {output:?}");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "onion-borrow-parity-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    // Preserve failed cases for diagnosis; remove successful runs below.
    let compiler = env!("CARGO_BIN_EXE_onion");
    let cases = [
        (
            r#"let mut x = 1 let r = &mut x *r = 25 println!("{}", x)"#,
            "25",
        ),
        (
            r#"let mut x = 1 let r = &mut x println!("{}", *r) x = 3 let s = &mut x *s = 4 println!("{}", x)"#,
            "1\n4",
        ),
        (
            r#"let mut x = 1 if true { let r = &mut x *r = 8 } println!("{}", x)"#,
            "8",
        ),
        (
            r#"let mut x = 1 loop { let r = &mut x *r = *r + 1 if *r == 3 { break } } println!("{}", x)"#,
            "3",
        ),
        (
            r#"let a = 1 let b = 2 let mut r = &a r = &b println!("{}", *r)"#,
            "2",
        ),
        (
            r#"let mut x = true let r = &mut x *r = false println!("{}", x)"#,
            "false",
        ),
        (
            r#"let mut x = 1.0 let r = &mut x *r = 2.0 println!("{}", x)"#,
            "2.0",
        ),
        (
            r#"let mut x = "a" let r = &mut x *r = "b" println!("{}", x)"#,
            "b",
        ),
    ];
    for (index, (body, expected)) in cases.into_iter().enumerate() {
        let source = dir.join(format!("case-{index}.onion"));
        let exe = dir.join(format!("case-{index}.exe"));
        fs::write(&source, format!("fn main() {{ {body} }}")).unwrap();
        let interpreted = output(
            Command::new(compiler)
                .arg("run")
                .arg(&source)
                .arg("--interp"),
        );
        assert!(interpreted.status.success(), "{interpreted:?}");
        let built = output(
            Command::new(compiler)
                .arg("build")
                .arg(&source)
                .arg("-o")
                .arg(&exe),
        );
        assert!(built.status.success(), "{built:?}");
        let native = output(&mut Command::new(&exe));
        assert!(native.status.success(), "{native:?}");
        let normalize = |bytes: &[u8]| {
            String::from_utf8_lossy(bytes)
                .replace("\r\n", "\n")
                .trim()
                .to_string()
        };
        assert_eq!(normalize(&interpreted.stdout), expected, "case {index}");
        assert_eq!(normalize(&native.stdout), expected, "case {index}");
    }
    fs::remove_dir_all(&dir).unwrap();
}
