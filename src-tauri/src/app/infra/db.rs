use rusqlite::{Connection, Result};

pub fn get_connection() -> Result<Connection> {
    // file DB akan dibuat di folder aplikasi
    Connection::open("kepegawaian.db")
}

pub fn init_db() -> Result<()> {
    let conn = get_connection()?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS employees (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nik TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            department TEXT NOT NULL,
            position TEXT NOT NULL,
            base_salary INTEGER NOT NULL,
            is_active INTEGER NOT NULL
        );
        ",
    )?;

    Ok(())
}
