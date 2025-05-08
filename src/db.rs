use rusqlite::{Connection, Result, params};
use crate::state::LOCAL_APP_STATE;

#[allow(unused)]
#[derive(Debug)]
pub struct Transaction {
    from_user: String,
    to_user: String,
    amount: f64,
    lamport_time: i64,
    source_node: String,
    optional_msg: Option<String>,
}


const NULL: &str = "NULL";

pub async fn init_db() -> Result<()> {
    {
        let state = LOCAL_APP_STATE.lock().await;
        state.connection.execute(
            "CREATE TABLE IF NOT EXISTS User (
            unique_name TEXT PRIMARY KEY,
            solde FLOAT NOT NULL
        )",
            [],
        )?;

        state.connection.execute(
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
    }

    log::debug!("Database initialized successfully.");
    Ok(())
}

pub async fn is_database_initialized() -> Result<bool> {
    {
        let state = LOCAL_APP_STATE.lock().await;
        let mut stmt = state.connection.prepare(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'Transactions')",
        )?;
        let exists: bool = stmt.query_row([], |row| row.get(0))?;
        Ok(exists)
    }
}

#[allow(unused)]
pub async fn drop_tables() -> Result<()> {
    {
        let state = LOCAL_APP_STATE.lock().await;
        state.connection.execute("DROP TABLE IF EXISTS Transactions;", [])?;
        state.connection.execute("DROP TABLE IF EXISTS User;", [])?;
    }
    log::debug!("Tables dropped successfully.");
    Ok(())
}

pub async fn user_exists(name: &str) -> Result<bool> {
    {
        let state = LOCAL_APP_STATE.lock().await;
        let mut stmt = state.connection.prepare("SELECT EXISTS(SELECT 1 FROM User WHERE unique_name = ?1)")?;
        let exists: bool = stmt.query_row(params![name], |row| row.get(0))?;
        Ok(exists)
    }
}

pub async fn create_user(unique_name: &str) -> Result<()> {
    if user_exists(unique_name).await? {
        log::warn!("User '{}' already exists.", unique_name);
        return Ok(());
    }
    {
        let state = LOCAL_APP_STATE.lock().await;
        state.connection.execute(
            "INSERT INTO User (unique_name, solde) VALUES (?1, 0)",
            params![unique_name],
        )?;
    }
    Ok(())
}

pub async fn calculate_solde(name: &str) -> Result<f64> {
    {
        let state = LOCAL_APP_STATE.lock().await;
        let mut stmt = state.connection.prepare(
            "SELECT
            IFNULL((SELECT SUM(amount) FROM Transactions WHERE to_user = ?1), 0) -
            IFNULL((SELECT SUM(amount) FROM Transactions WHERE from_user = ?1), 0)
        AS balance",
        )?;
        stmt.query_row(params![name], |row| row.get(0))
    }
}

pub async fn update_solde(name: &str) -> Result<()> {
    if !user_exists(name).await? {
        log::error!("User '{}' does not exist.", name);
        return Ok(());
    }
    let solde = calculate_solde(name).await?;
    {
        let state = LOCAL_APP_STATE.lock().await;
        state.connection.execute(
            "UPDATE User SET solde = ?1 WHERE unique_name = ?2",
            params![solde, name],
        )?;
    }
    Ok(())
}

pub async fn ensure_user(name: &str) -> Result<()> {
    if name != NULL && !user_exists(name).await? {
        create_user(name).await?;
    }
    Ok(())
}

pub async fn create_transaction(
    from_user: &str,
    to_user: &str,
    amount: f64,
    lamport_time: &mut i64,
    source_node: &str,
    optional_msg: &str,
) -> Result<()> {
    if from_user != NULL && calculate_solde(from_user).await? < amount {
        log::error!(
            "Insufficient funds: '{}' has less than {}.",
            from_user,
            amount
        );
        return Err(rusqlite::Error::InvalidQuery);
    }

    ensure_user(from_user).await?;
    ensure_user(to_user).await?;

    {
        let state = LOCAL_APP_STATE.lock().await;
        state.connection.execute(
            "INSERT INTO Transactions (from_user, to_user, amount, lamport_time, source_node, optional_msg)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![from_user, to_user, amount, *lamport_time, source_node, optional_msg],
        )?;
    }

    *lamport_time += 1;

    if from_user != NULL {
        update_solde(from_user).await?;
    }
    if to_user != NULL {
        update_solde(to_user).await?;
    }

    Ok(())
}

pub async fn deposit(
    user: &str,
    amount: f64,
    lamport_time: &mut i64,
    source_node: &str,
) -> Result<()> {
    if amount < 0.0 {
        log::error!("Negative deposit amount: {}", amount);
        return Err(rusqlite::Error::InvalidQuery);
    }
    if !user_exists(user).await?{
        return Err(rusqlite::Error::InvalidQuery);
    }
    create_transaction(
        NULL,
        user,
        amount,
        lamport_time,
        source_node,
        "Deposit",
    ).await
}

pub async fn withdraw(
    user: &str,
    amount: f64,
    lamport_time: &mut i64,
    source_node: &str,
) -> Result<()> {
    if amount < 0.0 {
        log::error!("Negative withdrawal amount: {}", amount);
        return Err(rusqlite::Error::InvalidQuery);
    }
    if !user_exists(user).await?{
        return Err(rusqlite::Error::InvalidQuery);
    }
    if calculate_solde(user).await?<amount{
        return Err(rusqlite::Error::InvalidQuery);
    }
    create_transaction(
        user,
        NULL,
        amount,
        lamport_time,
        source_node,
        "Withdraw",
    ).await
}

#[allow(unused)]
pub async fn create_user_with_solde(
    unique_name: &str,
    solde: f64,
    lamport_time: &mut i64,
    source_node: &str,
) -> Result<()> {
    create_user(unique_name).await?;
    create_transaction(
        NULL,
        unique_name,
        solde,
        lamport_time,
        source_node,
        "Initial deposit",
    ).await
}

pub async fn get_transaction(
    transac_time: i64,
    node: &str,
) -> Result<Option<Transaction>> {
    {
        let state = LOCAL_APP_STATE.lock().await;
        let mut stmt = state.connection.prepare(
            "SELECT from_user, to_user, amount, lamport_time, source_node, optional_msg
        FROM Transactions WHERE lamport_time = ?1 AND source_node = ?2",
        )?;

        match stmt.query_row(params![transac_time, node], |row| {
            Ok(Transaction {
                from_user: row.get(0)?,
                to_user: row.get(1)?,
                amount: row.get(2)?,
                lamport_time: row.get(3)?,
                source_node: row.get(4)?,
                optional_msg: row.get(5)?,
            })
        }) {
            Ok(tx) => Ok(Some(tx)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

pub async fn refund_transaction(
    transac_time: i64,
    node: &str,
    lamport_time: &mut i64,
    source_node: &str,
) -> Result<()> {
    if let Some(tx) = get_transaction(transac_time, node).await? {
        create_transaction(
            &tx.to_user,
            &tx.from_user,
            tx.amount,
            lamport_time,
            source_node,
            "Refund",
        ).await?;
    } else {
        log::error!(
            "No transaction found at time {} from node {}",
            transac_time,
            node
        );
    }
    Ok(())
}

pub async fn print_users() -> Result<()> {
    {
        let state = LOCAL_APP_STATE.lock().await;
        let mut stmt = state.connection.prepare("SELECT unique_name, solde FROM User")?;
        let users = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?;

        log::info!("-- Users --");
        for user in users {
            let (name, solde) = user?;
            log::info!("{}: {:.2}", name, solde);
        }
    }
    Ok(())
}

pub async fn print_transactions() -> Result<()> {
    {
        let state = LOCAL_APP_STATE.lock().await;
        let mut stmt = state.connection.prepare(
            "SELECT from_user, to_user, amount, lamport_time, source_node, optional_msg FROM Transactions",
        )?;
        let txs = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })?;

        log::info!("-- Transactions --");
        for tx in txs {
            let (from, to, amount, time, node, msg) = tx?;
            log::info!(
            "{} -> {} | {:.2} | time: {} | node: {} | msg: {:?}",
            from,
            to,
            amount,
            time,
            node,
            msg
        );
        }
    }
    Ok(())
}

pub async fn print_transaction_for_user(name: &str) -> Result<()> {
    {
        let state = LOCAL_APP_STATE.lock().await;
        let mut stmt = state.connection.prepare(
            "SELECT from_user, to_user, amount, lamport_time, source_node, optional_msg
        FROM Transactions WHERE from_user = ?1",
        )?;

        let txs = stmt.query_map(params![name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })?;

        log::info!("-- Transactions for user {} --", name);
        for tx in txs {
            let (from, to, amount, time, node, msg) = tx?;
            log::info!(
            "{} -> {} | {:.2} | time: {} | node: {} | msg: {:?}",
            from,
            to,
            amount,
            time,
            node,
            msg
        );
        }
    }
    Ok(())
}
