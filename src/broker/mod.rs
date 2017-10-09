//! The broker is a privileged controller/supervisor.
//!
//! Responsibilities include:
//! * Spawn the sandboxed process
//! * Host the IPC service
//! * Perform policy-allowed actions on behalf of the sandbox
use ipc;
use unix;
use util;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

fn tmpdir() -> Result<PathBuf, io::Error> {
    // Return tmpdir if it's absolute
    let tmpdir = env::temp_dir();
    if tmpdir.is_absolute() {
        return Ok(tmpdir);
    }

    // Otherwise, construct it from the current working directory
    match env::current_dir() {
        Ok(cwd) => Ok(cwd.join(tmpdir)),
        Err(e) => Err(e),
    }
}

fn create_tmpdir(suffix: &str) -> Result<String, io::Error> {
    // Get the system temporary directory
    let dir = match tmpdir() {
        Ok(val) => val,
        Err(e) => return Err(e),
    };

    // Construct name
    let path = dir.join(suffix);
    let filename = match path.to_str() {
        Some(val) => val,
        None => {
            let e = io::Error::new(io::ErrorKind::Other, "Error converting path to string");
            return Err(e)
        }
    };

    // Remove old directory if it exists
    // XXX: You can't run more than one instance at once because of this :(
    match fs::remove_dir_all(filename) { _ => () };

    // Create directory
    match fs::create_dir(filename) {
        Ok(_) => Ok(filename.to_string()),
        Err(e) => Err(e),
    }
}

pub fn main() {
    // Create tmpdir
    let dir = match create_tmpdir("sandpit.sandbox") {
        Ok(val) => val,
        Err(e) => {
            println!("[broker] Couldn't make temporary directory: {}", e);
            return;
        },
    };

    // Set tmpdir to cwd
    match env::set_current_dir(&dir[..]) {
        Ok(_) => (),
        Err(e) => {
            println!("[broker] Couldn't move into temporary directory: {}", e);
            return;
        },
    };

    // Spawn sandbox process
    // TODO: Supervise child and make sure it doesn't die. Will need to track the process as it
    //       forks. Maybe get the PID from the parent as it dies and create a future from waitpid
    use std::process::{Command, Stdio};
    let arg0 = unix::readlink("/proc/self/exe");
    let sandbox = match Command::new(arg0)
        .arg("--sandbox")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env_clear()
        .spawn() {
        Ok(val) => val,
        Err(e) => {
            println!("[broker] Couldn't spawn sandbox: {}", e);
            return;
        },
    };

    // Create demo
    let demo_filename = format!("{}/demo.txt", &dir[..]);
    match util::create_flag(&demo_filename[..], "not_a_flag") {
        Ok(_) => (),
        Err(e) => {
            println!("[broker] Couldn't make demo file: {}", e);
            return;
        },
    }

    // Create flag 0
    let flag0_filename = format!("{}/sandpit_flag0.txt", &dir[..]);
    match util::create_flag(&flag0_filename[..], "exit_light_enter_night") {
        Ok(_) => (),
        Err(e) => {
            println!("[broker] Couldn't make first flag: {}", e);
            return;
        },
    }

    // Create flag 2
    let flag2_filename = format!("{}/sandpit_flag2.txt", "/tmp");
    match util::create_flag(&flag2_filename[..], "somebody_just_took_a_sandwich") {
        Ok(_) => (),
        Err(e) => {
            println!("[broker] Couldn't make third flag: {}", e);
            return;
        },
    }

    // TODO: Make ipc::server a future
    ipc::server::start().unwrap();
    // TODO: Handle the sandbox dying/respawning/etc.
    // TODO: Add signal handling
    // TODO: Join all of the futures

    // Cleanup
    //match fs::remove_dir_all(&dir[..]) { _ => () };
    // TODO: Kill child processes
}
