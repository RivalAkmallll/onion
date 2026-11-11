use std::path::Path;
use std::process::Command;

/// Links an object file into an .exe using the MSVC linker.
#[cfg(windows)]
pub fn link(obj: &Path, exe: &Path) -> Result<(), String> {
    // `cc` finds link.exe and sets the LIB paths (even outside a VS prompt).
    let mut cmd = cc::windows_registry::find("x86_64-pc-windows-msvc", "link.exe")
        .unwrap_or_else(|| Command::new("link.exe"));

    cmd.arg("/NOLOGO")
        .arg("/SUBSYSTEM:CONSOLE")
        .arg(format!("/OUT:{}", exe.display()))
        .arg(obj)
        .args([
            "msvcrt.lib",
            "vcruntime.lib",
            "ucrt.lib",
            "legacy_stdio_definitions.lib",
            "kernel32.lib",
        ]);

    let output = cmd.output().map_err(|e| {
        format!(
            "could not start the MSVC linker (link.exe): {e}\n\
             help: install \"Build Tools for Visual Studio\" with the \
             \"Desktop development with C++\" workload"
        )
    })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "linker failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(not(windows))]
pub fn link(_obj: &Path, _exe: &Path) -> Result<(), String> {
    Err("native linking is only implemented for Windows (MSVC) in this phase".to_string())
}
