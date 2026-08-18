use std::process::{Command, Stdio};
use std::fs::File;
use std::path::Path;

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use crate::common;

fn run_gdb(gdb_path: &Path, gdb_script: &Path, pid: i32,logs_dir: &Path) -> std::io::Result<i32> {
    // Prepare logs directory
    let log_dir = logs_dir.join("gdb_output");
    std::fs::create_dir_all(&log_dir)?;

    // Log file path
    let log_path = log_dir.join(format!("{}_{}.log", pid, common::full_proc_name(pid).unwrap_or_else(|| "unknown".to_string())));
    let log_file = File::create(&log_path)?;

    // Build gdb command
    let mut cmd = Command::new(gdb_path);

    let gdb_ex_commands = [
        "set confirm off",
        "set pagination off",
        "set print pretty on",
        "set output-radix 16",
        "set disassembly-flavor intel",
        "set print elements 0",
    ];

    for ex in &gdb_ex_commands {
        cmd.arg("-ex").arg(ex);
    }

    cmd.arg("--command")
        .arg(gdb_script)
        .arg("-p")
        .arg(pid.to_string())
        .arg("-q")
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file));

    // Spawn GDB process
    let child = cmd.spawn()?;
    let gdb_pid = child.id().try_into().unwrap();

    debug!("Spawned gdb (pid={}) attached to target pid {}, logs at {:?}", gdb_pid, pid, log_path);

    Ok(gdb_pid)
}

pub fn attach_proc_name(proc_name: &str, gdb_script: &Path) {
    debug!("Mode: Local, Target Process Name: {}, GDB Script: {}", proc_name, gdb_script.display());
    match common::pidof(proc_name) {
        Some(pids) => {
            debug!("Resolved pids for {}: {}", proc_name, pids);
            attach_pids(&pids, gdb_script);
        }
        None => {
            error!("Could not resolve pids for process name: {}", proc_name);
        }
    }
}


pub fn attach_pids(pids: &str, gdb_script: &Path) {
    
    debug!("Mode: Local, Target PIDs: {}, GDB Script: {}", pids, gdb_script.display());
    
    // Ensure sessions directory exists
    if let Err(e) = std::fs::create_dir_all(common::SESSIONS_DIR.as_path()) {
        error!("Failed to create sessions directory at {:?}: {}", common::SESSIONS_DIR.as_path(), e);
        std::process::exit(1);
    }
    
    // Ensure current session directory exists
    if let Err(e) = std::fs::create_dir_all(common::CURR_SESSION_DIR.as_path()) {
        error!("Failed to create current session directory at {:?}: {}", common::CURR_SESSION_DIR.as_path(), e);
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
    
    let pid_list: Vec<i32> = pids.split(',').map(|s| s.trim().parse::<i32>().expect("Invalid PID")).collect();
    
    for pid in pid_list {
        let proc_name = common::full_proc_name(pid).unwrap_or_else(|| "unknown".to_string());
        match dbtracker::is_pid_attached(&conn, &pid) {
            Ok(gdb_pid) => {
                warn!("Process '{}' (pid={}) already being traced by GDB (pid={}). Skipping...", proc_name, pid, gdb_pid);
                continue;
            }
            Err(_db_err) => {
                // Not attached, proceed
            }
        }

        match run_gdb(common::GDB_PATH.as_path(), gdb_script, pid, common::CURR_SESSION_DIR.as_path()) {
            Ok(gdb_pid) => {
                let gdb_script_str = gdb_script.to_str().unwrap_or_else(|| "-");
                if let Err(e) = dbtracker::insert_entry(&conn, gdb_pid, pid, &proc_name, gdb_script_str, common::CURR_DATETIME.as_str()) {
                    error!("Failed to insert entry into database: {}", e);
                }
                info!("GDB (pid={}) attached to process '{}' (pid={})", gdb_pid, proc_name, pid);
            }
            Err(e) => {
                error!("Failed to run gdb for pid {}: {}", pid, e);
            }
        }
    
    }

    if let Some(script_name) = gdb_script.file_name() {
        let dest_path = common::CURR_SESSION_DIR.join(script_name);
        match std::fs::copy(gdb_script, &dest_path) {
            Ok(_) => debug!("Copied GDB script to {:?}", dest_path),
            Err(e) => error!("Failed to copy GDB script to {:?}: {}", dest_path, e),
        }
    }

    if let Ok(pids) = dbtracker::get_gdb_pids_by_session(&conn, common::CURR_SESSION_DIR.file_name().unwrap().to_str().unwrap()) {
        if !pids.is_empty() {
            let _ = dbtracker::show_session(&conn, common::CURR_SESSION_DIR.file_name().unwrap().to_str().unwrap());
            info!("To see logs: tail -f {:?}/gdb_output/*.log", common::CURR_SESSION_DIR.as_path());
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

    let gdb_pids = match dbtracker::get_gdb_pids_by_session(&conn, session) {
        Ok(pids) => pids,
        Err(e) => {
            error!("Failed to get GDB pids for session {}: {}", session, e);
            return;
        }
    };

    let total_pids = gdb_pids.len();
    let mut successful_kills = 0;
    let mut failed_kills = 0;

    for gdb_pid in gdb_pids {
        let pid = Pid::from_raw(gdb_pid);

        debug!("Sending SIGTERM to GDB pid {}", gdb_pid);

        match kill(pid, Signal::SIGTERM) {
            Err(e) => {
                // If process doesn't exist (ESRCH), remove it from database
                if e == nix::Error::ESRCH {
                    warn!("GDB process (pid={}) does not exist, removing from database", gdb_pid);
                    match dbtracker::remove_entry_by_gdb_pid(&mut conn, gdb_pid) {
                        Err(db_err) => error!("Failed to remove entry for GDB pid {} from database: {}", gdb_pid, db_err),
                        Ok(_) => {
                            debug!("Removed entry for GDB pid {} from database", gdb_pid);
                            successful_kills += 1;
                        }
                    }
                } else {
                    error!("Failed to send SIGTERM to pid {}: {}", gdb_pid, e);
                    failed_kills += 1;
                }
            }
            Ok(_) => {
                match dbtracker::remove_entry_by_gdb_pid(&mut conn, gdb_pid) {
                    Err(e) => error!("Failed to remove entry for GDB pid {} from database: {}", gdb_pid, e),
                    Ok(_) => {
                        debug!("Removed entry for GDB pid {} from database", gdb_pid);
                        successful_kills += 1;
                    }
                }
            }
        }
    }

    if total_pids > 0 {
        if successful_kills == 0 && failed_kills == total_pids {
            error!("Failed to terminate all {} GDB PIDs in session {}", total_pids, session);
        } else if successful_kills > 0 && successful_kills < total_pids {
            warn!("Terminated {} of {} GDB PIDs (failed to terminate {}) in session {}", successful_kills, total_pids, failed_kills, session);
        } else if successful_kills == total_pids {
            info!("Successfully terminated all {} GDB PIDs in session {}", total_pids, session);
        }
    } else {
        error!("No GDB PIDs found for session {}", session);
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

