use rusqlite::Connection;

/// Creates an in-memory SQLite database with the full schema initialized.
/// Use this in `#[cfg(test)]` modules to get a fresh DB for each test.
pub fn test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory DB");
    super::initialize(&conn).expect("Failed to initialize schema");
    conn
}
