use rusqlite::{Connection, Result};
use tabled::{Table, Tabled};
use std::path::Path;

#[derive(Tabled)]
struct ServerRow {
    id: i64,
    gdbserver_pid: i32,
    pid: i32,
    proc_name: String,
    port: u16,
    session: String,
}

pub fn open_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    Ok(conn)
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS server (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            gdbserver_pid       INTEGER NOT NULL,
            pid                 INTEGER NOT NULL,
            proc_name           TEXT NOT NULL,
            port                INTEGER NOT NULL,
            session             TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_server_pid
            ON server(pid);

        CREATE INDEX IF NOT EXISTS idx_server_gdbserver_pid
            ON server(gdbserver_pid);
        "#
    )?;
    Ok(())
}

pub fn insert_entry(
    conn: &Connection,
    gdbserver_pid: i32,
    pid: i32,
    proc_name: &str,
    port: u16,
    session: &str,
) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO server (gdbserver_pid, pid, proc_name, port, session)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        (gdbserver_pid, pid, proc_name, port, session),
    )?;
    Ok(())
}

pub fn remove_entry_by_gdbserver_pid(conn: &mut Connection, gdbserver_pid: i32) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM server WHERE gdbserver_pid = ?1",
        [gdbserver_pid],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn get_gdbserver_pids_by_session(conn: &Connection, session: &str) -> Result<Vec<i32>> {
    let mut stmt = conn.prepare(
        "SELECT gdbserver_pid FROM server WHERE session = ?1"
    )?;

    let gdbserver_pids_iter = stmt.query_map([session], |row| {
        row.get(0)
    })?;

    let mut gdbserver_pids = Vec::new();
    for gdbserver_pid_result in gdbserver_pids_iter {
        gdbserver_pids.push(gdbserver_pid_result?);
    }
    debug!("Found {} gdbserver_pids for session {}", gdbserver_pids.len(), session);
    Ok(gdbserver_pids)
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
        "SELECT id, gdbserver_pid, pid, proc_name, port, session
         FROM server
         WHERE session = ?1
         ORDER BY id ASC"
    )?;

    let rows = stmt.query_map([session], |row| {
        Ok(ServerRow {
            id: row.get(0)?,
            gdbserver_pid: row.get(1)?,
            pid: row.get(2)?,
            proc_name: row.get(3)?,
            port: row.get(4)?,
            session: row.get(5)?,
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

pub fn get_sessions(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT session FROM server ORDER BY session ASC"
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

