use crate::model::account::Account;
use crate::utils::DB;

pub fn fetch_accounts_paginated(page: Option<usize>, page_size: Option<usize>) -> (Vec<Account>, usize) {
    use crate::utils::DB;
    let db = DB.lock().unwrap();
    if let Some(conn) = &*db {
        println!("🟡 Querying accounts with pagination...");
        
        // First get total count
        let total_count: usize = conn.prepare("SELECT COUNT(*) FROM accounts")
            .and_then(|mut stmt| stmt.query_row([], |row| Ok(row.get::<_, i64>(0)? as usize)))
            .unwrap_or(0);
        
        // Helper function to create Account from row
        let create_account = |row: &rusqlite::Row| -> rusqlite::Result<Account> {
            Ok(Account {
                id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
                description: row.get::<_, String>(2)?,
                access_key: row.get::<_, String>(3)?,
                secret_key: row.get::<_, String>(4)?,
                is_default: row.get::<_, i64>(5).map(|e| e == 1)?,
                default_region: row.get::<_, String>(6)?
            })
        };
        
        // Execute appropriate query
        let accounts = if let (Some(page_num), Some(size)) = (page, page_size) {
            let offset = page_num * size;
            let mut stmt = conn.prepare("SELECT id, name, description, access_key, secret_key, is_default, default_region FROM accounts ORDER BY id LIMIT ? OFFSET ?")
                .expect("prepare failed");
            stmt.query_map([size as i64, offset as i64], create_account)
                .expect("Failed to query accounts")
                .filter_map(Result::ok)
                .collect()
        } else {
            let mut stmt = conn.prepare("SELECT id, name, description, access_key, secret_key, is_default, default_region FROM accounts ORDER BY id")
                .expect("prepare failed");
            stmt.query_map([], create_account)
                .expect("Failed to query accounts")
                .filter_map(Result::ok)
                .collect()
        };

        (accounts, total_count)
    } else {
        (vec![], 0)
    }
}

pub fn fetch_accounts() -> Vec<Account> {
    let (accounts, _) = fetch_accounts_paginated(None, None);
    accounts
}

pub fn save_account_to_db(
    account_id: Option<i64>,
    name: &str,
    description: &str,
    access_key: &str,
    secret_key: &str,
    is_default: bool,
    default_region: &str,
) {
    if let Some(conn) = DB.lock().unwrap().as_ref() {
        //not transactional but good enough for now
        if is_default {
            // Unset is_default for all other accounts
            conn.execute(
                "UPDATE accounts SET is_default = 0 WHERE is_default = 1",
                [],
            ).expect("Failed to unset previous default account");
        }

        if let Some(id) = account_id {
            println!("UPDATING ACCOUNT {}", id);
            conn.execute(
                "UPDATE accounts SET name = ?1, description = ?2, access_key = ?3, secret_key = ?4, is_default = ?5, default_region = ?6 WHERE id = ?7",
                rusqlite::params![name, description, access_key, secret_key, if is_default { 1 } else { 0 }, default_region, id],
            ).expect("Failed to update account");
        } else {
            println!("INSERTING NEW ACCOUNT");
            conn.execute(
                "INSERT INTO accounts (name, description, access_key, secret_key, is_default, default_region) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![name, description, access_key, secret_key, if is_default { 1 } else { 0 }, default_region],
            ).expect("Failed to insert account");
        }
    }
}

pub fn get_default_account() -> Option<Account> {
    let db = DB.lock().unwrap();
    let conn = db.as_ref()?;
    
    let mut stmt = conn.prepare(
        "SELECT id, name, description, access_key, secret_key, is_default, default_region 
         FROM accounts 
         WHERE is_default = 1 
         ORDER BY id DESC 
         LIMIT 1"
    ).ok()?;
    
    let account = stmt.query_row([], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            access_key: row.get(3)?,
            secret_key: row.get(4)?,
            is_default: row.get::<_, i64>(5).map(|v| v == 1)?,
            default_region: row.get(6)?
        })
    }).ok();
    
    if let Some(ref acc) = account {
        println!("Found default account: {:?}", acc);
    } else {
        println!("No default account found");
    }
    
    account
}

pub fn delete_account(account_id: i64) -> () {
    let db = DB.lock().unwrap();
    if let Some(conn) = &*db {
        conn.execute("DELETE FROM accounts WHERE id = ?", [&account_id])
            .expect("Failed to delete account");
    }
}