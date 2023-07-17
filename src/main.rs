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

/// Used to locate binaries. Why? See comments inside [content].
#[allow(dead_code)]
async fn content_locate_binaries() -> String {
    let free = run("whereis", |prog| {
        prog.arg("free");
    });
    let df = run("whereis", |prog| {
        prog.arg("df");
    });
    let (free, df) = (free.await, df.await);
    "".to_owned() + &free + "\n" + &df
}

/// Content returned over HTTP.
async fn content() -> String {
    // Beware: Some Unix distributions (at least Manjaro, possibly Arch, too) have aliases set (for
    // example in ~/.bashrc). Those prettify the output, but are not available under non-personal
    // accounts, such as daemons/web services! Hence we use full paths to executables. (That may
    // make this not portable to other OS'es, but that doesn't matter.)
    //
    // To complicate, Manjaro has free & df under both /usr/bin & /bin. But: Shuttle.rs does NOT
    // have /usr/bin/df - only /bin/df.
    //
    // If your Linux or Mac OS doesn't support the following locations, and you can figure out how
    // to determine it, feel free to file a pull request.
    let free = run("/usr/bin/free", |prog| {
        prog.arg("-m");
    });
    let tmpfs = run("/bin/df", |prog| {
        prog.arg("-m").arg("/tmp");
    });
    let (free, tmpfs) = (free.await, tmpfs.await);
    "Sysinfo of (free tier) Shuttle.rs. Thank you Shuttle & Love you.\n".to_owned()
        + "Format and URL routing/handling are subject to change!\n"
        + "(https://github.com/peter-kehl/sys-info.shuttleapp.rs)\n\n"
        + "free -m:\n"
        + &free
        + "\n-----\n\n"
        + "df -m /tmp:\n"
        + &tmpfs
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    assert!(cfg!(target_os = "linux"), "For Linux only.");

    //let router = Router::new().route("/", get(content));
    let router = Router::new().route("/", get(content));

    Ok(router.into())
}
