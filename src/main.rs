//! sandpit is a toy sandboxed Linux application

#![warn(box_pointers,
        fat_ptr_transmutes,
        missing_debug_implementations,
        trivial_casts,
        unsafe_code,
        unstable_features,
        unused_extern_crates,
        unused_import_braces,
        unused_qualifications,
        unused_results,
        variant_size_differences)]

extern crate libc;
extern crate futures;
#[macro_use]
extern crate tokio_core;
extern crate tokio_timer;
extern crate tokio_uds;
#[macro_use]
extern crate nom;

mod broker;
mod ipc;
mod sandbox;
mod unix;
mod util;

use std::env;

enum ProcessType {
    Broker,
    Sandbox,
}

fn parse_arg(arg: &str) -> Option<ProcessType> {
    match arg {
        "--sandbox" => Some(ProcessType::Sandbox),
        _ => None,
    }
}

fn parse_args() -> Option<ProcessType> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => Some(ProcessType::Broker),
        2 => parse_arg(&args[1][..]),
        _ => None,
    }
}

fn help() {
    println!("sandpit 1.0.0");
    println!("Lachlan Bishop <toor@lxb.io>\n");
    println!("sandpit is a toy sandboxed Linux application\n");
    println!("Project home page: https://github.com/0x6c7862/sandpit\n");
    println!("USAGE:");
    println!("	sandpit");
    println!("	sandpit --sandbox\n");
    println!("OPTIONS:");
    println!("	--sandbox");
    println!("		Spawn the sandbox process. Used by the broker.")
}

fn main() {
    // NOTE: Don't worry, I know I shouldn't be hand rolling this part :p
    let parsed_args = parse_args();
    let process = match parsed_args {
        Some(val) => val,
        None => return help(),
    };

    match process {
        ProcessType::Broker => broker::main(),
        ProcessType::Sandbox => sandbox::main(),
    }
}
