use std::process::{Command, Stdio};
use std::path::Path;
use std::io::Read;
use std::time::Duration;
use std::thread::sleep;

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use nix::errno::Errno;

use crate::common;

pub fn run_gdbserver(gdbserver_path: &Path, pid: i32, port: u16) -> std::io::Result<i32> {

    // Build gdbserver command
    let mut cmd = Command::new(gdbserver_path);

    // Capture stderr so we can detect immediate failures such as "Operation not permitted"
    cmd.arg("--attach").arg(format!("0.0.0.0:{}", port)).arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    // Spawn gdbserver server process
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            // If the underlying OS error is EPERM (Operation not permitted), return a PermissionDenied
            if let Some(code) = e.raw_os_error() {
                if Errno::from_i32(code) == Errno::EPERM {
                    return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied,
                        format!("Operation not permitted when attaching to pid {}: {}", pid, e)));
                }
            }
            return Err(e);
        }
    };

    // Give gdbserver a small window to fail immediately (e.g. when it cannot attach due to EPERM).
    // If it exits quickly, read stderr and return a helpful error. If it stays running, assume success.
    let mut stderr_output = String::new();
    let mut stderr_handle = child.stderr.take();

    // Poll for up to 500ms
    for _ in 0..10 {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Child exited already; try to read stderr
                if let Some(mut s) = stderr_handle.take() {
                    let _ = s.read_to_string(&mut stderr_output);
                }

                // If stderr indicates permission problems, map to PermissionDenied
                if stderr_output.contains("Operation not permitted") || stderr_output.contains("Permission denied") {
                    return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied,
                        format!("gdbserver failed to attach to pid {}: {}", pid, stderr_output.trim())));
                }

                // Otherwise return a generic error containing the exit status and stderr
                return Err(std::io::Error::new(std::io::ErrorKind::Other,
                    format!("gdbserver exited immediately (status={}) stderr: {}", status, stderr_output.trim())));
            }
            Ok(None) => {
                // still running, assume attached; give it a bit more time
                sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other,
                    format!("Failed to poll gdbserver process: {}", e)));
            }
        }
    }

    // If we reach here, the child is still running -> treat as successful start
    let gdbserver_pid = child.id().try_into().unwrap();
    debug!("Spawned gdbserver (pid={}) attached to target pid {} on port {}", gdbserver_pid, pid, port);
    Ok(gdbserver_pid)
}

pub fn attach_proc_name(proc_name: Option<&str>, init_listen_port: u16) {
    if let Some(proc_name) = proc_name {
        debug!("Mode: Server, Target Process Name: {}", proc_name);
        match common::pidof(proc_name) {
            Some(pids) => {
                debug!("Resolved pids for {}: {}", proc_name, pids);
                attach_pids(Some(&pids), init_listen_port);
            },
            None => {
                error!("No running processes found with name: {}", proc_name);
            }
        }
    } else {
        error!("No process name provided to attach to.");
    }
}

pub fn attach_pids(pids: Option<&str>, init_listen_port: u16) {

    let mut gdbserver_lport: u16 = init_listen_port;

    // Ensure sessions directory exists
    if let Err(e) = std::fs::create_dir_all(common::SESSIONS_DIR.as_path()) {
        error!("Failed to create sessions directory at {:?}: {}", common::SESSIONS_DIR.as_path(), e);
        std::process::exit(1);
    }

    let conn = match dbtracker::open_db(common::GDBORCH_DB.as_path()) {
        Ok(conn) => {
            debug!("Successfully opened database");
            if let Err(e) = dbtracker::init_schema(&conn) {
                error!("Failed to initialize schema: {}", e);
                std::process::exit(1);
            }
            conn
        },
        Err(e) => {
            error!("Failed to open DB at {:?}: {}", common::GDBORCH_DB.as_path(), e);
            std::process::exit(1); 
        }
    };

    if let Some(pids) = pids {
        debug!("Mode: Server, Target PIDs: {}", pids);
        let pid_list: Vec<i32> = pids.split(',').map(|s| s.trim().parse::<i32>().expect("Invalid PID")).collect();
        for pid in pid_list {
            let proc_name = common::full_proc_name(pid).unwrap_or_else(|| "unknown".to_string());
            while !is_port_available(gdbserver_lport) && gdbserver_lport < 65535 {
                warn!("Port {} is already in use, trying {} instead", gdbserver_lport, gdbserver_lport + 1);
                gdbserver_lport += 1;
            }
            match run_gdbserver(common::GDBSERVER_PATH.as_path(), pid, gdbserver_lport) {
                Ok(gdbserver_pid) => {
                    if let Err(e) = dbtracker::insert_entry(&conn, gdbserver_pid, pid, &proc_name, gdbserver_lport, common::CURR_SESSION_DIR.file_name().unwrap().to_str().unwrap()) {
                        error!("Failed to insert entry into database: {}", e);
                    }
                    info!("Started gdbserver (pid={}) for target pid {} ({}) on port {}", gdbserver_pid, pid, proc_name, gdbserver_lport);
                    gdbserver_lport += 1;
                },
                Err(e) => {
                    error!("Failed to start gdbserver for pid {} ({}): {}", pid, proc_name, e);
                }
            }
        }
    } else {
        error!("No PIDs provided to attach to.");
    }

    if let Ok(pids) = dbtracker::get_gdbserver_pids_by_session(&conn, common::CURR_SESSION_DIR.file_name().unwrap().to_str().unwrap()) {
        if !pids.is_empty() {
            let _ = dbtracker::show_session(&conn, common::CURR_SESSION_DIR.file_name().unwrap().to_str().unwrap());
        }
    }
}

pub fn show_all_sessions() {
    let conn = match dbtracker::open_db(common::GDBORCH_DB.as_path()) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to open DB at {:?}: {}", common::GDBORCH_DB.as_path(), e);
            return;
        }
    };
    debug!("Successfully opened database for showing sessions");
    dbtracker::show_all_sessions(&conn).unwrap_or_else(|e| {
        error!("Failed to list sessions from database: {}", e);
    });
}

pub fn stop_session(session: &str) {
    let mut conn = match dbtracker::open_db(common::GDBORCH_DB.as_path()) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to open DB at {:?}: {}", common::GDBORCH_DB.as_path(), e);
            return;
        }
    };

    debug!("Successfully opened database for stopping session {}", session);

    let gdbserver_pids = match dbtracker::get_gdbserver_pids_by_session(&conn, session) {
        Ok(pids) => pids,
        Err(e) => {
            error!("Failed to get gdbserver pids for session {}: {}", session, e);
            return;
        }
    };

    let total_pids = gdbserver_pids.len();
    let mut successful_kills = 0;
    let mut failed_kills = 0;

    for gdbserver_pid in gdbserver_pids {
        let pid = Pid::from_raw(gdbserver_pid);

        debug!("Sending SIGTERM to gdbserver pid {}", gdbserver_pid);

        match kill(pid, Signal::SIGTERM) {
            Err(e) => {
                // If process doesn't exist (ESRCH), remove it from database
                if e == nix::Error::ESRCH {
                    warn!("GDB process (pid={}) does not exist, removing from database", gdbserver_pid);
                    match dbtracker::remove_entry_by_gdbserver_pid(&mut conn, gdbserver_pid) {
                        Err(db_err) => error!("Failed to remove entry for gdbserver pid {} from database: {}", gdbserver_pid, db_err),
                        Ok(_) => {
                            debug!("Removed entry for gdbserver pid {} from database", gdbserver_pid);
                            successful_kills += 1;
                        }
                    }
                } else {
                    error!("Failed to send SIGTERM to pid {}: {}", gdbserver_pid, e);
                    failed_kills += 1;
                }
            }
            Ok(_) => {
                match dbtracker::remove_entry_by_gdbserver_pid(&mut conn, gdbserver_pid) {
                    Err(e) => error!("Failed to remove entry for gdbserver pid {} from database: {}", gdbserver_pid, e),
                    Ok(_) => {
                        debug!("Removed entry for gdbserver pid {} from database", gdbserver_pid);
                        successful_kills += 1;
                    }
                }
            }
        }
    }

    if total_pids > 0 {
        if successful_kills == 0 && failed_kills == total_pids {
            error!("Failed to terminate all {} gdbserver PIDs in session {}", total_pids, session);
        } else if successful_kills > 0 && successful_kills < total_pids {
            warn!("Terminated {} of {} gdbserver PIDs (failed to terminate {}) in session {}", successful_kills, total_pids, failed_kills, session);
        } else if successful_kills == total_pids {
            info!("Successfully terminated all {} gdbserver PIDs in session {}", total_pids, session);
        }
    } else {
        error!("No gdbserver PIDs found for session {}", session);
    }
}

pub fn stop_all_sessions() {
    let conn = match dbtracker::open_db(common::GDBORCH_DB.as_path()) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to open DB at {:?}: {}", common::GDBORCH_DB.as_path(), e);
            return;
        }
    };

    debug!("Successfully opened database for stopping all sessions");

    match dbtracker::get_sessions(&conn) {
        Ok(sessions) => {
            if sessions.is_empty() {
                error!("No sessions to stop.");
            }
            for session in sessions {
                stop_session(&session);
            }
        }
        Err(e) => {
            error!("Failed to get sessions from database: {}", e);
        }
    }
}

fn is_port_available(port: u16) -> bool {
    match std::net::TcpListener::bind(("0.0.0.0", port)) {    
        Ok(bind_addr) => {
            //unbind the tcp port after testing
            drop(bind_addr);
            true
        },
        Err(_) => false,
    }
}