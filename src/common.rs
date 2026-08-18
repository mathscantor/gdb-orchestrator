use once_cell::sync::Lazy;
use std::env;
use std::path::PathBuf;
use chrono::prelude::*;
use std::fs;
use std::process::Command;

pub static CURRENT_DIR: Lazy<PathBuf> = Lazy::new(|| {
    env::current_exe()
        .expect("failed to get exe path")
        .parent()
        .expect("failed to get parent")
        .to_path_buf()
});

pub static SESSIONS_DIR: Lazy<PathBuf> = Lazy::new(|| {
    CURRENT_DIR.join(".gdborch")
});

pub static GDBORCH_DB: Lazy<PathBuf> = Lazy::new(|| {
    SESSIONS_DIR.join("gdborch.db")
});

pub static CURR_DATETIME: Lazy<String> = Lazy::new(|| {
    let now: DateTime<Local> = Local::now();
    format!("{:02}-{:02}-{:04}T{:02}-{:02}-{:02}", now.day(), now.month(), now.year(), now.hour(), now.minute(), now.second())
});

pub static CURR_SESSION_DIR: Lazy<PathBuf> = Lazy::new(|| {
    SESSIONS_DIR.join(CURR_DATETIME.as_str())
});

pub static GDB_PATH: Lazy<PathBuf> = Lazy::new(|| {
    // Use GDB_PATH environment variable when provided, otherwise default to /usr/bin/gdb
    match env::var("GDB_PATH") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => PathBuf::from("/usr/bin/gdb"),
    }
});

pub static GDBSERVER_PATH: Lazy<PathBuf> = Lazy::new(|| {
    // Use GDBSERVER_PATH environment variable when provided, otherwise default to /usr/bin/gdbserver
    match env::var("GDBSERVER_PATH") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => PathBuf::from("/usr/bin/gdbserver"),
    }
});


pub fn pidof(proc_name: &str) -> Option<String> {
    let output = Command::new("pidof")
        .arg(proc_name)
        .output()
        .ok()?; // return None if command fails

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pids: Vec<&str> = stdout.split_whitespace().collect();
    if pids.is_empty() {
        None
    } else {
        Some(pids.join(","))
    }
}

pub fn full_proc_name(pid: i32) -> Option<String> {
    let path: String = format!("/proc/{}/cmdline", pid);
    let data = fs::read(path).ok()?;

    if data.is_empty() {
        return None;
    }

    // cmdline is NUL-separated; argv[0] is the executable
    let first = data.split(|b| *b == 0).next()?;

    let full = String::from_utf8(first.to_vec()).ok()?;

    // Optional: strip path, keep just the binary name
    let name = full
        .rsplit('/')
        .next()
        .unwrap_or(&full)
        .to_string();

    Some(name)
}

