use crate::model::account::Account;
use crate::utils::DB;

pub fn fetch_accounts() -> Vec<Account> {
    use crate::utils::DB;
    let db = DB.lock().unwrap();
    if let Some(conn) = &*db {
        println!("🟡 Querying accounts...");
        let mut stmt = conn.prepare("SELECT id, name, description, access_key, secret_key, is_default, default_region FROM accounts")
            .expect("prepare failed");
        let account_iter = stmt
            .query_map([], |row| {
                Ok(Account {
                    id: row.get::<_, i64>(0)?,
                    name: row.get::<_, String>(1)?,
                    description: row.get::<_, String>(2)?,
                    access_key: row.get::<_, String>(3)?,
                    secret_key: row.get::<_, String>(4)?,
                    is_default: row.get::<_, i64>(5).map(|e| e == 1)?,
                    default_region: row.get::<_, String>(6)?
                })
            })
            .expect("Failed to query accounts");

        account_iter.filter_map(Result::ok).collect()
    } else {
        vec![]
    }
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
    let db_guard = crate::utils::DB.lock().unwrap();
    let conn = db_guard.as_ref()?;

    let mut stmt = conn.prepare("SELECT id, name, description, access_key, secret_key, is_default, default_region FROM accounts ORDER BY is_default DESC, id ASC LIMIT 1")
        .ok()?;
    let mut rows = stmt.query([]).ok()?;
    if let Some(row) = rows.next().ok()? {
        let acc = Account {
            id: row.get::<_, i64>(0).ok()?,
            name: row.get::<_, String>(1).ok()?,
            description: row.get::<_, String>(2).ok()?,
            access_key: row.get::<_, String>(3).ok()?,
            secret_key: row.get::<_, String>(4).ok()?,
            is_default: row.get::<_, i64>(5).map(|e| e == 1).ok()?,
            default_region: row.get::<_, String>(5).ok()?
        };
        println!("returning some acc: {:?}", acc);
        Some(acc)
    } else {
        println!("returning none");
        None
    }
}

pub fn delete_account(account_id: i64) {
    let db = DB.lock().unwrap();
    if let Some(conn) = &*db {
        conn.execute("DELETE FROM accounts WHERE id = ?", [&account_id])
            .expect("Failed to delete account");
    }
}