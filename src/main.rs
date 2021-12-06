#![feature(associated_type_defaults)]
#![feature(termination_trait_lib)]

mod config;
mod exit;
mod getent;

use clap::Parser;
use command_fds::{CommandFdExt, FdMapping};
use std::{fs::File, os::unix::io::AsRawFd, process::Command};

use config::ToBwrapArgs;
use exit::Exit;

/// A dumb automatic sandbox launcher.
#[derive(Debug, Parser)]
#[clap(version = "0.1.0", author = "Dan Nixon (dan-nixon.com)")]
struct Options {
    /// Program to be launched in sandbox
    program: String,

    /// Arguments to be passed to sandboxed program
    args: Vec<String>,
}

fn main() -> Exit {
    let opts = Options::parse();

    let conf_file = match File::open("sandy.yml") {
        Ok(f) => f,
        Err(e) => return Exit::Me(format!("failed to read config file: {}", e)),
    };

    let conf: config::Configuration = match serde_yaml::from_reader(conf_file) {
        Ok(c) => c,
        Err(e) => return Exit::Me(format!("failed to parse config file: {}", e)),
    };

    let profile = if let Some(profile) = conf.get_profile_for_program(&opts.program) {
        profile
    } else {
        return Exit::Me(format!(
            "failed to get profile for program \"{}\"",
            &opts.program
        ));
    };

    let etc_passwd = match &profile.config.users {
        Some(u) => match getent::lookup(getent::Database::Passwd, u) {
            Ok(c) => Some(c),
            Err(e) => return Exit::Me(e),
        },
        None => None,
    };

    let etc_group = match &profile.config.groups {
        Some(g) => match getent::lookup(getent::Database::Group, g) {
            Ok(c) => Some(c),
            Err(e) => return Exit::Me(e),
        },
        None => None,
    };

    let mut bwrap_args = Vec::<&str>::default();
    let mut fd_mappings = Vec::<FdMapping>::default();

    if let Some(f) = &etc_passwd {
        fd_mappings.push(FdMapping {
            parent_fd: f.as_raw_fd(),
            child_fd: 10,
        });
        bwrap_args.append(&mut vec!["--file", "10", "/etc/passwd"]);
    }

    if let Some(f) = &etc_group {
        fd_mappings.push(FdMapping {
            parent_fd: f.as_raw_fd(),
            child_fd: 11,
        });
        bwrap_args.append(&mut vec!["--file", "11", "/etc/group"]);
    }

    let mut child = match Command::new("bwrap")
        .arg("--die-with-parent")
        .args(profile.config.bwrap_args())
        .args(bwrap_args)
        .arg("--")
        .arg(opts.program)
        .args(opts.args)
        .fd_mappings(fd_mappings)
        .unwrap()
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return Exit::Me(e.to_string()),
    };

    match child.wait() {
        Ok(exit) => {
            if let Some(code) = exit.code() {
                Exit::Program(code)
            } else {
                Exit::Me("unknown exit code".to_string())
            }
        }
        Err(e) => Exit::Me(e.to_string()),
    }
}
