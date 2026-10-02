//! Integration test: runs the compiled game binary and checks what it prints.
//!
//! Integration tests live in `tests/` and see the package only from the outside, the way a
//! user does. This one becomes obsolete once the game opens a window (M6), because running
//! the binary would then block until the window is closed.

use std::process::Command;

#[test]
fn prints_the_engine_greeting() {
    // Cargo builds the binary before running integration tests and passes its path here.
    let output = Command::new(env!("CARGO_BIN_EXE_platformer"))
        .output()
        .expect("failed to run the platformer binary");

    assert!(output.status.success(), "exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), engine::greeting());
}
