//! Miscellaneous system related functionality.
//!
//! Other than the extern portion of this module, this is basically a really poorly done
//! https://github.com/nix-rust/nix clone. Use that instead :)

#![allow(unsafe_code)]
use libc;
use std::fs;
use std::ffi::{CString, CStr};
use std::ptr;
use std::env;
// XXX: I'm mixing two different UnixDatagram types by not being async everywhere , but it doesn't
//      matter because I don't even deal with the server code properly here :/
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixDatagram;

mod ffi {
    extern {
        pub fn sandbox() -> i32;
        pub fn open_recvmsg(socket: i32, fd: &i32) -> i32;
        pub fn open_sendmsg(socket: i32, fd: i32) -> i32;
        pub fn open_sendmsg_err(socket: i32) -> i32;
    }
}

// TODO: Should probably make a macro out of the "call and check errno" pattern
// FIXME: The way I'm calling all them is different; some I set errno some I don't, etc.

pub enum ProcessType {
    Parent,
    Child,
}

// FIXME: Use Result not Option
pub fn fork() -> Option<ProcessType> {
    // XXX: I'm not actually 100% sure whether this is safe to call at all in a Rust program.
    let ret;
    unsafe {
        let errno = libc::__errno_location();
        *errno = 0;
        ret = libc::fork();
        if ret < 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("fork(): {:?}", err);
            return None;
        }
    }

    // XXX: Wait isn't this backwards? The PIDs seem right in `ps` but I don't know what the child
    //      is :/
    if ret == 0 {
        Some(ProcessType::Child)
    } else {
        Some(ProcessType::Parent)
    }
}

// FIXME: Handle errors
pub fn readlink(path: &str) -> String {
    // XXX: This is _terrible_! I told you not to look in this file :)
    unsafe {
        let len: usize = 256;
        let path_c = CString::new(path).unwrap();
        let mut buf: Vec<u8> = Vec::with_capacity(len);
        let p = buf.as_mut_ptr() as *mut libc::c_char;
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::readlink(path_c.as_ptr(), p, len);

        // On error just return the original argument
        if ret < 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("Error readlink(): {:?}", err);
            return path.to_string();
        } else if ret == len as isize {
            println!("Error readlink(): Buffer was too small. Path truncated");
        }

        // Null terminate at ret
        buf.set_len(len);
        buf[ret as usize] = 0;

        // Shorten buf to first null
        let s = CStr::from_ptr(p);
        buf.set_len(s.to_bytes().len());

        // Create a string and return it
        let ret = CString::new(buf).unwrap();
        ret.into_string().unwrap()
    }
}

// FIXME: Handle errors
pub fn set_hostname(hostname: &str) {
    unsafe {
        let hostname_c = CString::new(hostname).unwrap();
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::sethostname(hostname_c.as_ptr(), hostname.len());
        if ret != 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("[foo] Error setting hostname: {:?}", err); // FIXME: lol
        }
    }
}

// FIXME: Use Result not Option
// FIXME: Does libc give better types than this? i32 seems wrong for an fd regardless.
pub fn open_recvmsg(socket: &UnixDatagram) -> Option<i32> {
    unsafe {
        let socketfd = socket.as_raw_fd();
        let mut fd = -1;
        let ret = ffi::open_recvmsg(socketfd, &mut fd);
        let errno = *libc::__errno_location();
        if ret == -127 {
            println!("[ipc client] Request failed policy");
            None
        } else if ret < 0 || errno != 0 || fd < 0 {
            let err = CStr::from_ptr(libc::strerror(errno));
            println!("[ipc client] Error calling recvmsg(): ({:?}) {:?}", errno, err);
            None
        } else {
            Some(fd)
        }
    }
}

// FIXME: Use Result not Option
pub fn open_sendmsg(socket: i32, fd: i32) -> Option<()> {
    unsafe {
        let ret = ffi::open_sendmsg(socket, fd);
        let errno = *libc::__errno_location();
        if ret < 0 || errno != 0 {
            let err = CStr::from_ptr(libc::strerror(errno));
            println!("[ipc server] Error calling sendmsg(): ({:?}) {:?}", errno, err);
            None
        } else {
            Some(())
        }
    }
}

// FIXME: Use Result not Option
pub fn open_sendmsg_err(socket: i32) -> Option<()> {
    unsafe {
        let ret = ffi::open_sendmsg_err(socket);
        let errno = *libc::__errno_location();
        if ret < 0 || errno != 0 {
            let err = CStr::from_ptr(libc::strerror(errno));
            println!("[ipc server] Error calling sendmsg(): ({:?}) {:?}", errno, err);
            None
        } else {
            Some(())
        }
    }
}

// FIXME: Handle errors
// FIXME: This name is misleading and doesn't actually discribe behaviour
pub fn prctl() {
    unsafe {
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        if ret != 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("[broker] Error enabling prctl: {:?}", err);
        }
    }
}

// FIXME: Handle errors
pub fn dropcap() {
    unsafe {
        // NOTE: This just blindly calls prctl() assuming there are 38 incrementing values, which
        //       at the time of writing is true against modern kernel versions
        for cap in 0..37 {
            let errno = libc::__errno_location();
            *errno = 0;
            let ret = libc::prctl(libc::PR_CAPBSET_DROP, cap, 0, 0, 0);
            if ret != 0 || *errno != 0 {
                let err = CStr::from_ptr(libc::strerror(*errno));
                println!("[broker] Error dropping capability: ({}) {:?}", cap, err);
            }
        }
    }
}

// FIXME: Shared code
// FIXME: Handle errors
pub fn unshare(flag: i32) {
    unsafe {
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::syscall(libc::SYS_unshare, flag);
        if ret != 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("[broker] Error enabling namespaces: {:?}", err);
        }
    }
}

pub fn get_uid() -> libc::uid_t {
    unsafe {
        libc::getuid()
    }
}

pub fn get_euid() -> libc::uid_t {
    unsafe {
        libc::geteuid()
    }
}

pub fn get_gid() -> libc::uid_t {
    unsafe {
        libc::getgid()
    }
}

pub fn get_egid() -> libc::uid_t {
    unsafe {
        libc::getegid()
    }
}

// TODO: mount()
// FIXME: Should return an option
//fn mount(source, target, fstype, flags: u64) { }

// TODO: umount()
// FIXME: Should return an option
//fn umount(source, target, fstype, flags: u64) { }

// TODO: pivot_root()
// FIXME: Should return an option
//fn pivot_root(source, target, fstype, flags: u64) { }

// TODO: Handle errors
pub fn pivot() {
    unsafe {
        let source = ptr::null();
        let target = CString::new("/").unwrap();
        let fstype = ptr::null();
        let flags = libc::MS_REC | libc::MS_PRIVATE;
        let data = ptr::null();
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::mount(source, target.as_ptr(), fstype, flags, data);
        if ret != 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("[sandbox] private root mount(): ({:?}) {:?}", *errno, err);
        }
    }

    unsafe {
        let source = CString::new(".").unwrap();
        let target = CString::new(".").unwrap();
        let fstype = ptr::null();
        let flags = libc::MS_REC | libc::MS_BIND;
        let data = ptr::null();
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::mount(source.as_ptr(), target.as_ptr(), fstype, flags, data);
        if ret != 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("[sandbox] bind self mount(): ({:?}) {:?}", *errno, err);
        }
    }

    // FIXME: Hardcoded dir
    env::set_current_dir("/tmp/sandpit.sandbox").unwrap();

    fs::create_dir("proc").unwrap();
    unsafe {
        let source = CString::new("none").unwrap();
        let target = CString::new("proc").unwrap();
        let fstype = CString::new("proc").unwrap();
        let flags = libc::MS_NOEXEC | libc::MS_NOSUID | libc::MS_NODEV;
        let data = ptr::null();
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::mount(source.as_ptr(), target.as_ptr(), fstype.as_ptr(), flags, data);
        if ret != 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("[sandbox] proc mount(): ({:?}) {:?}", *errno, err);
        }
    }

    fs::create_dir("sys").unwrap();
    unsafe {
        let source = CString::new("none").unwrap();
        let target = CString::new("sys").unwrap();
        let fstype = CString::new("sysfs").unwrap();
        let flags = libc::MS_NOEXEC | libc::MS_NOSUID | libc::MS_NODEV | libc::MS_RDONLY;
        let data = ptr::null();
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::mount(source.as_ptr(), target.as_ptr(), fstype.as_ptr(), flags, data);
        if ret != 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("[sandbox] sys mount(): ({:?}) {:?}", *errno, err);
        }
    }

    unsafe {
        let new_root = CString::new(".").unwrap();
        let put_old = CString::new(".").unwrap();
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::syscall(libc::SYS_pivot_root, new_root.as_ptr(), put_old.as_ptr());
        if ret != 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("[sandbox] pivot_root(): ({:?}) {:?}", *errno, err);
        }
    }

    unsafe {
        let target = CString::new(".").unwrap();
        let errno = libc::__errno_location();
        *errno = 0;
        let ret = libc::umount2(target.as_ptr(), libc::MNT_DETACH);
        if ret != 0 || *errno != 0 {
            let err = CStr::from_ptr(libc::strerror(*errno));
            println!("[sandbox] umount2(): ({:?}) {:?}", *errno, err);
        }
    }

    env::set_current_dir("/").unwrap();
}

// FIXME: Handle errors
pub fn seccomp() {
    unsafe {
        let ret = ffi::sandbox();
        let errno = *libc::__errno_location();
        if ret != 0 || errno != 0 {
            let err = CStr::from_ptr(libc::strerror(errno));
            println!("[sandbox] Error enabling seccomp: {:?}", err);
        }
    }
}
