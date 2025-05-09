#[derive(Debug)]
pub struct Transaction {
    from_user: String,
    to_user: String,
    amount: f64,
    lamport_time: i64,
    source_node: String,
    optional_msg: Option<String>,
}

#[cfg(feature = "server")]
pub fn init_db(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    use rusqlite::{Connection, Result, params};
    conn.execute(
        "CREATE TABLE IF NOT EXISTS User (
            unique_name TEXT PRIMARY KEY,
            solde FLOAT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Transactions (
            from_user TEXT,
            to_user TEXT NOT NULL,
            amount FLOAT NOT NULL,
            lamport_time INTEGER NOT NULL,
            source_node TEXT NOT NULL,
            optional_msg TEXT,
            FOREIGN KEY(from_user) REFERENCES User(unique_name),
            FOREIGN KEY(to_user) REFERENCES User(unique_name),
            PRIMARY KEY(lamport_time, source_node)
        )",
        [],
    )?;

    println!("Database initialized successfully.");
    Ok(())
}

#[cfg(feature = "server")]
pub fn is_database_initialized(conn: &rusqlite::Connection) -> rusqlite::Result<bool> {
    use rusqlite::{Connection, Result, params};
    let mut stmt = conn.prepare(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'Transactions')",
    )?;
    let exists: bool = stmt.query_row([], |row| row.get(0))?;
    Ok(exists)
}

#[cfg(feature = "server")]
pub fn print_users(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("SELECT unique_name, solde FROM User")?;
    let users = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?;

    println!("-- Users --");
    for user in users {
        let (name, solde) = user?;
        println!("{}: {:.2}", name, solde);
    }
    Ok(())
}

#[cfg(feature = "server")]
pub fn get_users(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT unique_name FROM User")?;
    let users = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut users = users.map(|user| user.unwrap()).collect::<Vec<_>>();
    users.sort();
    Ok(users)
}

#[cfg(feature = "server")]
pub fn add_user(conn: &rusqlite::Connection, unique_name: &str) -> rusqlite::Result<()> {
    use rusqlite::params;
    if user_exists(conn, unique_name)? {
        tracing::warn!("User '{}' already exists.", unique_name);
        return Ok(());
    }
    conn.execute(
        "INSERT INTO User (unique_name, solde) VALUES (?1, 0)",
        params![unique_name],
    )?;
    Ok(())
}

#[cfg(feature = "server")]
pub fn user_exists(conn: &rusqlite::Connection, name: &str) -> rusqlite::Result<bool> {
    use rusqlite::params;
    let mut stmt = conn.prepare("SELECT EXISTS(SELECT 1 FROM User WHERE unique_name = ?1)")?;
    let exists: bool = stmt.query_row(params![name], |row| row.get(0))?;
    Ok(exists)
}
