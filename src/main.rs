#![allow(non_snake_case)]
use clap::Parser;
use dioxus::prelude::*;

use views::{Home, User};

mod control;
mod data;
mod views;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = std::process::id() as u64)]
    site_id: u64,
    #[arg(long, default_value_t = 0)]
    port: u16,
    #[arg(long, value_delimiter = ',')]
    peers: Vec<String>,
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    ip: String,
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/user/:id")]
    User { id: String },
}

const FAVICON: Asset = asset!("/assets/logo.png");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const LOW_PORT: u16 = 11000;
    const HIGH_PORT: u16 = 12000;

    use data::{AppState, GLOBAL_APP_STATE};
    use std::io::Write;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let args = Args::parse();

    // Get the address the backend should run on. If the CLI is running, the CLI proxies fullstack into the main address
    // and we use the generated address the CLI gives us
    let address = dioxus::cli_config::fullstack_address_or_localhost();

    // Set up the axum router for interaction between the frontend and the backend
    let router = axum::Router::new()
        // You can add a dioxus application to the router with the `serve_dioxus_application` method
        // This will add a fallback route to the router that will serve your component and backend functions
        .serve_dioxus_application(ServeConfigBuilder::default(), App);

    let router = router.into_make_service();
    let backend_listener = tokio::net::TcpListener::bind(address).await.unwrap();

    // Instanciate our database connection and check if it's initialized
    let conn: rusqlite::Connection = rusqlite::Connection::open("peillute.db").unwrap();
    if !control::is_database_initialized(&conn)? {
        let _ = control::init_db(&conn);
    }

    // setup peer to peer networking
    let port_range = LOW_PORT..=HIGH_PORT;
    let mut selected_port = args.port;
    if selected_port == 0 {
        for port in port_range {
            if let Ok(listener) = std::net::TcpListener::bind(("127.0.0.1", port)) {
                selected_port = port;
                drop(listener);
                break;
            }
        }
    }
    let site_ip: &str = &args.ip;
    let local_addr: std::net::SocketAddr = format!("{}:{}", site_ip, selected_port).parse()?;

    println!("Elected address: {}", local_addr);

    {
        let mut state = GLOBAL_APP_STATE.lock().await;
        state.site_id = args.site_id;
        state.local_addr = local_addr;
        state.nb_sites_on_network = args.peers.len();
        state.vector_clock = (0..args.peers.len())
            .map(|_| std::sync::atomic::AtomicU64::new(0))
            .collect();
        state.lamport_clock = std::sync::atomic::AtomicU64::new(args.site_id);
    }

    control::announce(site_ip, LOW_PORT, HIGH_PORT).await;

    let network_listener_local_addr = local_addr.clone();
    let peer_listener: tokio::net::TcpListener =
        tokio::net::TcpListener::bind(network_listener_local_addr).await?;

    let node_name = "A"; // TODO :should be in app state and define by discovery
    let mut local_lamport_time: i64 = 0; // TODO :should use app state

    let stdin: tokio::io::Stdin = tokio::io::stdin();
    let reader: tokio::io::BufReader<tokio::io::Stdin> = tokio::io::BufReader::new(stdin);
    let mut lines: tokio::io::Lines<BufReader<tokio::io::Stdin>> = reader.lines();

    println!("Welcome on peillute, write /help to get the command list.");
    print!("> ");
    std::io::stdout().flush().unwrap();

    let main_loop_app_state = GLOBAL_APP_STATE.clone();
    let _ = main_loop(
        main_loop_app_state,
        &mut lines,
        &conn,
        &mut local_lamport_time,
        node_name,
        peer_listener,
    )
    .await;

    // Finally, we can launch the backend
    axum::serve(backend_listener, router).await.unwrap();

    Ok(())
}

#[cfg(feature = "server")]
async fn main_loop(
    _state: std::sync::Arc<tokio::sync::Mutex<crate::data::AppState>>,
    lines: &mut tokio::io::Lines<tokio::io::BufReader<tokio::io::Stdin>>,
    conn: &rusqlite::Connection,
    local_lamport_time: &mut i64,
    node_name: &str,
    listener: tokio::net::TcpListener,
) {
    loop {
        tokio::select! {
            Ok((stream, addr)) = listener.accept() => {
                let _ = control::start_listening(stream, addr).await;
            }
            _ = tokio::signal::ctrl_c() => {
                // disconnect().await;
                println!("👋 Bye !");
                std::process::exit(0);
            }
        }
    }
}

// #[cfg(feature = "server")]
// async fn disconnect() {
//     use crate::data::LOCAL_APP_STATE;
//     // lock just to get the local address and site id
//     let (local_addr, site_id, peer_addrs, clock) = {
//         let state = LOCAL_APP_STATE.lock().await;
//         (
//             state.get_local_addr().to_string(),
//             state.get_site_id().to_string(),
//             state.get_peers(),
//             state.get_clock().clone(),
//         )
//     };

//     {
//         let mut state = LOCAL_APP_STATE.lock().await;
//         state.increment_lamport();
//         state.increment_vector_current();
//     }

//     println!("Shutting down site {}.", site_id);
//     for peer_addr in peer_addrs {
//         let peer_addr_str = peer_addr.to_string();
//         {
//             // Before sending the message, we need to update the local clock
//             let mut state = LOCAL_APP_STATE.lock().await;
//             state.increment_lamport();
//             state.increment_vector_current();
//         }
//         if let Err(e) = network::send_message(
//             &peer_addr_str,
//             message::MessageInfo::None,
//             None,
//             message::NetworkMessageCode::Disconnect,
//             &local_addr,
//             &site_id,
//             clock.clone(),
//         )
//         .await
//         {
//             println!("Error sending message to {}: {}", peer_addr_str, e);
//         }
//     }
// }

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Router::<Route> {}
    }
}
