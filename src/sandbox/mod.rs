//! The sandbox is an unprivileged task runner
//!
//! Responsibilities include:
//! * Drop privileges
//! * Execute code
//! * Delegate actions to the broker through an IPC client
use ipc;
use libc;
use unix;
use util;
use futures::*;
use std::fs;
use std::io::prelude::*;
use std::mem;
use std::time::*;
use tokio_timer::*;

// FIXME: Handle errors
fn map_user(uid: u32, gid: u32) {
    let mut uid_file = fs::OpenOptions::new().write(true).open("/proc/self/uid_map").unwrap();
    let uid_map = format!("{0}	{0}	1\n", uid);
    uid_file.write_all(uid_map.as_bytes()).unwrap();

    let mut setgroups_file = fs::OpenOptions::new().write(true).open("/proc/self/setgroups").unwrap();
    setgroups_file.write_all(b"deny").unwrap();

    let mut gid_file = fs::OpenOptions::new().write(true).open("/proc/self/gid_map").unwrap();
    let gid_map = format!("{0}	{0}	1\n", gid);
    gid_file.write_all(gid_map.as_bytes()).unwrap();
}

// FIXME: Should be up to caller to ignore error behaviour
fn check_uid() -> (u32, u32) {
    let uid = unix::get_uid();
    let gid = unix::get_gid();
    let euid = unix::get_euid();
    let egid = unix::get_egid();
    if uid == 0 || euid == 0 || gid == 0 || egid == 0 {
        println!("[broker] Warning: Don't run me as root");
    }
    (uid, gid)
}

// FIXME: Handle errors
fn check_unpriv_clone() {
    // Get the value of /proc/sys/kernel/unprivileged_userns_clone
    let unpriv_clone = match fs::File::open("/proc/sys/kernel/unprivileged_userns_clone") {
        // FIXME: This is clumsy
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => Some(contents),
                Err(_) => None,
            }
        }
        Err(_) => None,
    };

    // Warn the user if it is 0
    let warning = "[broker] Warning: /proc/sys/kernel/unprivileged_userns_clone set to 0. \
                   Creating namespaces as a user will fail. \
                   Set the value to 1 using the following command: \
                   sudo sysctl kernel.unprivileged_userns_clone=1";
    match unpriv_clone {
        Some(val) => match &val[..] {
            "0" => println!("{}", warning),
            _ => (),
        },
        None => (),
    };
}

fn sleep(timer: &Timer, duration: Duration) {
    let sleep = timer.sleep(duration);
    sleep.wait().unwrap();
}

// FIXME: Handle errors
// FIXME: Unsafe
fn read_fd(fd: i32) {
    use std::os::unix::io::FromRawFd;

    let mut file;
    unsafe {
        file = fs::File::from_raw_fd(fd);
    }
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    print!("Recieved file contents: {}", contents)
}

pub fn main() {
    // Create flag 1
    println!("[+] Writing /tmp/sandpit_flag1.txt");
    let flag1 = match util::create_flag("/tmp/sandpit_flag1.txt", "this_is_sandpit_turtle") {
        Ok(val) => val,
        Err(e) => {
            println!("[sandbox] Couldn't make second flag: {}", e);
            return;
        },
    };
    mem::forget(flag1);  // Whoops! :)

    // Prevent children inheriting privileges
    unix::prctl();

    // Create namespaces
    // FIXME: Handle errors gracefully
    // TODO: This should probably be broken up a bit more
    check_unpriv_clone();
    let (uid, gid) = check_uid();
    unix::unshare(libc::CLONE_NEWUSER
        | libc::CLONE_NEWNS
        | libc::CLONE_NEWPID
        | libc::CLONE_NEWCGROUP
        | libc::CLONE_NEWIPC
        | libc::CLONE_NEWNET
        | libc::CLONE_NEWUTS
    );
    match unix::fork().unwrap() {
        unix::ProcessType::Parent => return,
        unix::ProcessType::Child => (),
    };
    map_user(uid, gid);
    unix::set_hostname("sandpit");
    unix::pivot();

    // Wait until successful connection to the IPC
    // TODO: Hang until ready if the sandbox dir isn't present
    // TODO: This should be its own future
    let timer = Timer::default();
    let mut client;
    loop {
        match ipc::client::connect() {
            Some(val) => {
                client = val;
                break;
            },
            None => sleep(&timer, Duration::from_secs(1))
        }
    }

    // Enable seccomp and drop privileges
    // FIXME: Handle errors gracefully
    unix::seccomp();
    unix::dropcap();

    // NOTE: Sandboxed code begins here

    // Read an allowed file through the IPC
    println!("[sandbox] Reading an allowed file from /tmp/sandpit.sandbox/demo.txt");
    match client.open("/tmp/sandpit.sandbox/demo.txt") {
        Some(val) => read_fd(val),
        None => println!("[sandbox] Something went wrong opening"),
    }

    // Read a denied file through the IPC
    println!("[sandbox] Reading a denied file from /etc/passwd");
    match client.open("/etc/passwd") {
        Some(val) => read_fd(val),
        None => println!("[sandbox] Something went wrong opening"),
    }

    // Send pings forever
    loop {
        println!("[sandbox] Sending ping");
        match client.ping() {
            Some(_) => println!("[sandbox] Got pong"),
            None => println!("[sandbox] Something went wrong pinging"),
        };
        sleep(&timer, Duration::from_secs(30));
    }
}
