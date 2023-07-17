use axum::{routing::get, Router};
//use std::process::{ExitStatus, Output};
use tokio::process::Command;

/// Crate a new [Command] based on `program`. Set it to `kill_on_drop`.
fn command(program: &'static str) -> Command {
    let mut command = Command::new(program);
    command.kill_on_drop(true);
    command
}

fn ascii_bytes_to_string(bytes: Vec<u8>) -> String {
    let mut result = String::with_capacity(bytes.len());
    for byte in bytes {
        result.push(char::from(byte));
    }
    result
}

/// Start the program, with any arguments or other adjustments done in `modify` closure. Kill on drop.
///
/// On success, return the program's output, treated as ASCII.
async fn run<F: Fn(&mut Command)>(program: &'static str, modify: F) -> String {
    let mut command = command(program);
    modify(&mut command);
    let out = command
        .output()
        .await
        .unwrap_or_else(|err| panic!("Expected to run {program}, but failed abruptly: {err}"));
    assert!(
        out.status.success(),
        "Expecting {program} to succeed, but it failed: {}",
        ascii_bytes_to_string(out.stderr)
    );
    ascii_bytes_to_string(out.stdout)
}

/// Content returned over HTTP.
async fn content() -> String {
    let free = run("free", |_| ());
    let tmpfs = run("df", |prog| {
        prog.arg("-m").arg("/tmp");
    });
    let (free, tmpfs) = (free.await, tmpfs.await);
    format!("{free}\n{tmpfs}")
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    assert!(cfg!(target_os = "linux"), "For Linux only.");

    let router = Router::new().route("/", get(content));

    Ok(router.into())
}
