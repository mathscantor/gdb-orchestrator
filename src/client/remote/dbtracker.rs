use rusqlite::{Connection, Result};
use tabled::{Table, Tabled};
use std::path::Path;

#[derive(Tabled)]
struct ClientRemoteRow {
    id: i64,
    gdb_pid: i32,
    rhost_rport: String,
    gdb_script: String,
    session: String,
}

pub fn open_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    Ok(conn)
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS client_remote (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            gdb_pid         INTEGER NOT NULL,
            rhost_rport     TEXT NOT NULL,
            gdb_script      TEXT,
            session         TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_client_remote_rhost_rport
            ON client_remote(rhost_rport);

        CREATE INDEX IF NOT EXISTS idx_client_remote_gdb_pid
            ON client_remote(gdb_pid);
        "#
    )?;
    Ok(())
}

pub fn insert_entry(
    conn: &Connection,
    gdb_pid: i32,
    rhost_rport: &str,
    gdb_script: &str,
    session: &str,
) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO client_remote (gdb_pid, rhost_rport, gdb_script, session)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        (gdb_pid, rhost_rport, gdb_script, session),
    )?;
    Ok(())
}

pub fn remove_entry_by_gdb_pid(conn: &mut Connection, gdb_pid: i32) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM client_remote WHERE gdb_pid = ?1",
        [gdb_pid],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn get_gdb_pids_by_session(conn: &Connection, session: &str) -> Result<Vec<i32>> {
    let mut stmt = conn.prepare(
        "SELECT gdb_pid FROM client_remote WHERE session = ?1"
    )?;

    let gdb_pids_iter = stmt.query_map([session], |row| {
        row.get(0)
    })?;

    let mut gdb_pids = Vec::new();
    for gdb_pid_result in gdb_pids_iter {
        gdb_pids.push(gdb_pid_result?);
    }
    debug!("Found {} gdb_pids for session {}", gdb_pids.len(), session);
    Ok(gdb_pids)
}

pub fn show_all_sessions(conn: &Connection) -> Result<()> {
    let sessions = get_sessions(conn)?;

    if sessions.is_empty() {
        info!("No sessions found.");
        return Ok(());
    }

    for session in sessions {
        show_session(conn, &session)?;
    }

    Ok(())
}

pub fn show_session(conn: &Connection, session: &str) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, gdb_pid, rhost_rport, gdb_script, session
         FROM client_remote
         WHERE session = ?1
         ORDER BY id ASC"
    )?;

    let rows = stmt.query_map([session], |row| {
        Ok(ClientRemoteRow {
            id: row.get(0)?,
            gdb_pid: row.get(1)?,
            rhost_rport: row.get(2)?,
            gdb_script: row.get(3)?,
            session: row.get(4)?,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }

    if entries.is_empty() {
        info!("No entries found for session {}.", session);
        return Ok(());
    }

    info!("=== Session: {} ===", session);
    println!("{}", Table::new(entries));

    Ok(())
}

pub fn is_rhost_rport_attached(conn: &Connection, rhost_rport: &str) -> Result<i32> {
    let mut stmt = conn.prepare(
        "SELECT gdb_pid FROM client_remote WHERE rhost_rport = ?1"
    )?;

    let gdb_pid: i32 = stmt.query_row([rhost_rport], |row| {
        row.get(0)
    })?;

    Ok(gdb_pid)
}

pub fn get_sessions(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT session FROM client_remote ORDER BY session ASC"
    )?;

    let session_iter = stmt.query_map([], |row| {
        row.get(0)
    })?;

    let mut sessions = Vec::new();
    for session_result in session_iter {
        sessions.push(session_result?);
    }

    Ok(sessions)
}