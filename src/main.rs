#![allow(non_snake_case)]
use dioxus::prelude::*;

mod control;

// The entry point for the server
#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    // Get the address the server should run on. If the CLI is running, the CLI proxies fullstack into the main address
    // and we use the generated address the CLI gives us
    let address = dioxus::cli_config::fullstack_address_or_localhost();

    // Set up the axum router
    let router = axum::Router::new()
        // You can add a dioxus application to the router with the `serve_dioxus_application` method
        // This will add a fallback route to the router that will serve your component and server functions
        .serve_dioxus_application(ServeConfigBuilder::default(), App);

    // Finally, we can launch the server
    let router = router.into_make_service();
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

// For any other platform, we just launch the app
#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}
#[component]
fn App() -> Element {
    let mut users = use_signal(|| Vec::new());
    let mut user_input = use_signal(|| "".to_string());

    use_future(move || async move {
        if let Ok(data) = get_users().await {
            users.set(data);
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0",
        }
        body {
            div { class: "app-container",
                div { id: "home-page", class: "page active",
                    header { class: "app-header",
                        a { class: "logo-link", href: "#",
                            img {
                                class: "logo-img",
                                src: asset!("/assets/logo.png"),
                                alt: "Logo peillute",
                            }
                        }
                        h1 { "Peillute" }
                        button { id: "theme-toggle", class: "theme-button",
                            text { "🌙" }
                        }
                    }
                    main {
                        h2 { "Users" }
                        div { class: "user-list-container",
                            ul {
                                for item in users.iter() {
                                    li { "{item}" }
                                }
                            }
                        }
                    }
                    footer { class: "add-user-footer",
                        input {
                            r#type: "text",
                            id: "new-user-name",
                            placeholder: "New user name",
                            value: user_input,
                            oninput: move |event| user_input.set(event.value()),
                        }
                        button {
                            id: "add-user-button",
                            onclick: move |_| async move {
                                if let Ok(_) = add_user(user_input.to_string()).await {
                                    user_input.set("".to_string());
                                }
                                if let Ok(data) = get_users().await {
                                    users.set(data);
                                }
                            },
                            text { "✔️" }
                        }
                    }
                }
            }
        }
    }
}

#[server]
async fn get_users() -> Result<Vec<String>, ServerFnError> {
    let conn = rusqlite::Connection::open("peillute.db")?;
    let users = control::get_users(&conn)?;
    Ok(users)
}

#[server]
async fn add_user(name: String) -> Result<(), ServerFnError> {
    let conn = rusqlite::Connection::open("peillute.db")?;
    control::add_user(&conn, &name)?;
    println!("User added with name {}", name);
    Ok(())
}
