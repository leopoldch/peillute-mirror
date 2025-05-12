#[cfg(feature = "server")]
pub fn run_cli(
    line: Result<Option<String>, std::io::Error>,
    conn: &rusqlite::Connection,
    mut local_lamport_time: &mut i64,
    node_name: &str,
) -> u8 {
    use std::io::{self as std_io, Write};
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

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
fn handle_command(cmd: Command, conn: &rusqlite::Connection, lamport_time: &mut i64, node: &str) {
    use crate::control;
    match cmd {
        Command::CreateUser => {
            let name = prompt("Username");
            control::add_user(conn, &name).unwrap();
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

        _ => {
            println!("❓ Unknown command - error cli");
        }
    }
}

#[cfg(feature = "server")]
fn prompt(label: &str) -> String {
    use std::io::{self as std_io, Write};
    let mut input = String::new();
    print!("{label} > ");
    std_io::stdout().flush().unwrap();
    std_io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[cfg(feature = "server")]
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
