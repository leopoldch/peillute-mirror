use super::db;
use rusqlite::Connection;
use std::io::{self as std_io, Write};

pub fn run_cli(
    line: Result<Option<String>, std::io::Error>,
    conn: &Connection,
    mut local_lamport_time: &mut i64,
    node_name: &str,
) -> u8 {
    match line {
        Ok(Some(cmd)) => {
            let command = parse_command(&cmd);
            handle_command(command, conn, &mut local_lamport_time, node_name);
            print!("> ");
            std_io::stdout().flush().unwrap();
            0
        }
        Ok(None) => {
            println!("Aucun input");
            0
        }
        Err(e) => {
            tracing::error!("Erreur de lecture stdin : {}", e);
            1
        }
    }
}

enum Command {
    CreateUser,
    UserAccounts,
    PrintUserTransactions,
    PrintTransactions,
    Deposit,
    Withdraw,
    Transfer,
    Pay,
    Refund,
    Help,
    Unknown(String),
}

fn parse_command(input: &str) -> Command {
    match input.trim() {
        "/create_user" => Command::CreateUser,
        "/user_accounts" => Command::UserAccounts,
        "/print_user_tsx" => Command::PrintUserTransactions,
        "/print_tsx" => Command::PrintTransactions,
        "/deposit" => Command::Deposit,
        "/withdraw" => Command::Withdraw,
        "/transfer" => Command::Transfer,
        "/pay" => Command::Pay,
        "/refund" => Command::Refund,
        "/help" => Command::Help,
        other => Command::Unknown(other.to_string()),
    }
}

fn handle_command(cmd: Command, conn: &Connection, lamport_time: &mut i64, node: &str) {
    match cmd {
        Command::CreateUser => {
            let name = prompt("Username");
            db::create_user(conn, &name).unwrap();
        }

        Command::UserAccounts => {
            db::print_users(conn).unwrap();
        }

        Command::PrintUserTransactions => {
            let name = prompt("Username");
            db::print_transaction_for_user(conn, &name).unwrap();
        }

        Command::PrintTransactions => {
            db::print_transactions(conn).unwrap();
        }

        Command::Deposit => {
            let name = prompt("Username");
            let amount = prompt_parse::<f64>("Deposit amount");
            db::deposit(conn, &name, amount, lamport_time, node).unwrap();
        }

        Command::Withdraw => {
            let name = prompt("Username");
            let amount = prompt_parse::<f64>("Withdraw amount");
            db::withdraw(conn, &name, amount, lamport_time, node).unwrap();
        }

        Command::Transfer => {
            let name = prompt("Username");
            let amount = prompt_parse::<f64>("Transfer amount");
            let _ = db::print_users(conn);
            let beneficiary = prompt("Beneficiary");
            db::create_transaction(conn, &name, &beneficiary, amount, lamport_time, node, "")
                .unwrap();
        }

        Command::Pay => {
            let name = prompt("Username");
            let amount = prompt_parse::<f64>("Payment amount");
            db::create_transaction(conn, &name, "NULL", amount, lamport_time, node, "").unwrap();
        }

        Command::Refund => {
            let name = prompt("Username");
            db::print_transaction_for_user(conn, &name).unwrap();
            let transac_time = prompt_parse::<i64>("Lamport time");
            let transac_node = prompt("Node");
            db::refund_transaction(conn, transac_time, &transac_node, lamport_time, node).unwrap();
        }

        Command::Help => {
            println!("📜 Command list:");
            println!("/create_user      - Create a personal account");
            println!("/user_accounts    - List all users");
            println!("/print_user_tsx   - Show a user’s transactions");
            println!("/print_tsx        - Show all system transactions");
            println!("/deposit          - Deposit money to an account");
            println!("/withdraw         - Withdraw money from an account");
            println!("/transfer         - Transfer money to another user");
            println!("/pay              - Make a payment (to NULL)");
            println!("/refund           - Refund a transaction");
        }

        Command::Unknown(cmd) => {
            println!("❓ Unknown command: {}", cmd);
        }
    }
}

fn prompt(label: &str) -> String {
    let mut input = String::new();
    print!("{label} > ");
    std_io::stdout().flush().unwrap();
    std_io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn prompt_parse<T: std::str::FromStr>(label: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    loop {
        let input = prompt(label);
        match input.parse::<T>() {
            Ok(value) => break value,
            Err(_) => println!("Invalid input. Try again."),
        }
    }
}
