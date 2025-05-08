use std::path::PathBuf;
use directories::ProjectDirs;

use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::sync::Mutex;
use dioxus::prelude::{GlobalSignal, Signal};
use crate::model::account::Account;
use crate::repositories::account_repo::fetch_accounts;
use std::sync::Once;

static INIT: Once = Once::new();

pub static DB: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));
pub static CURRENT_ACCOUNT: GlobalSignal<Option<Account>> = Signal::global(|| None);

pub fn init_state() {
    INIT.call_once(|| {
        let accounts = fetch_accounts();
        if let Some(account) = accounts.iter().find(|a| a.is_default) {
            CURRENT_ACCOUNT.write().insert(account.clone());
        }
    });
}

pub fn init_db() {
    let db_path = get_data_dir().join("dios3").join("accounts.db");
    let db_dir = db_path.parent().unwrap();

    // Add a debug check
    println!("Ensuring DB directory exists: {}", db_dir.display());

    // Explicitly create everything in the parent path
    std::fs::create_dir_all(db_dir).expect("Failed to create DB directory");

    assert!(db_dir.exists(), "DB directory does not exist after create_dir_all");
    println!("DB Directory: {}", db_dir.display());

    let conn = Connection::open(&db_path).expect("Failed to open SQLite database");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            access_key TEXT,
            secret_key TEXT,
            is_default INTEGER,
            default_region TEXT
        )",
        [],
    ).expect("Failed to create accounts table");

    *DB.lock().unwrap() = Some(conn);
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "softberries", env!("CARGO_PKG_NAME"))
}

/// Gets the user specified data directory
/// Eventually takes the system default location
pub fn get_data_dir() -> PathBuf {
    let PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    let DATA_FOLDER: Option<PathBuf> = std::env::var(format!("{}_DATA", PROJECT_NAME.clone())).ok().map(PathBuf::from);
    let directory = if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    };
    directory
}