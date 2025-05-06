mod db;

#[cfg(feature = "server")]
pub use db::{add_user, get_users, init_db, is_database_initialized};
