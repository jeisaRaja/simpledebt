use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Error, Result};

#[derive(Debug)]
pub struct User {
    id: i32,
    username: String,
    balance: i64,
}

#[derive(Debug)]
pub struct Transactions {
    id: i32,
    user_id: i32,
    transaction_type: String,
    amount: i64,
    date: DateTime<Local>,
    description: String,
}

impl User {
    fn new(username: String) -> User {
        return User {
            id: 0,
            username,
            balance: 0,
        };
    }
}

pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn new() -> DB {
        let conn = connect_to_db().unwrap();
        return DB { conn };
    }

    pub fn select_person(&self, name: &String) -> Result<User, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, username, balance FROM users WHERE username = ?1")?;
        let user = stmt.query_row([&name], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                balance: row.get(2)?,
            })
        })?;
        Ok(user)
    }

    pub fn create_person(
        &self,
        name: &String,
        balance: u64,
        transaction_type: String,
        description: Option<String>,
    ) {
        let _user = self.conn.execute(
            "INSERT INTO users (username, balance) VALUES (?1,?2) RETURNING id",
            params![name, balance],
        );
        if balance != 0 {
            let user_id = self.conn.last_insert_rowid();
            let date_now = Local::now().to_string();
            let _ = self.conn.execute(
                "INSERT INTO transactions (user_id, transaction_type, amount, date, description)",
                params![
                    user_id,
                    transaction_type,
                    balance,
                    date_now,
                    description
                ],
            );
        };
        print!("creating user\n");
    }

    pub fn give_to(
        &self,
        name: &String,
        balance: &u64,
        transaction_type: String,
        description: Option<String>,
    ) {
        let _give = self
            .conn
            .execute(
                "UPDATE users SET balance = balance + ?1 WHERE username = ?2 ",
                params![balance, name],
            )
            .expect("Failed to update user balance");
        self.insert_transaction(balance, transaction_type, description)
    }

    pub fn receive_from(
        &self,
        name: &String,
        balance: &u64,
        transaction_type: String,
        description: Option<String>,
    ) {
        let _ = self
            .conn
            .execute(
                "UPDATE users SET balance = balance - ?1 WHERE username = ?2 ",
                params![balance, name],
            )
            .expect("Failed to update user balance");
        self.insert_transaction(balance, transaction_type, description)
    }

    fn insert_transaction(
        &self,
        balance: &u64,
        transaction_type: String,
        description: Option<String>,
    ) {
        let date_now = Local::now().to_string();
        let user_id = self.conn.last_insert_rowid();
        let _ = self.conn.execute(
            "INSERT INTO transactions 
        (user_id, transaction_type, amount, date, description) 
     VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, transaction_type, balance, date_now, description],
        );
    }
}

fn connect_to_db() -> Result<Connection, Error> {
    let db_path = "utang.db";
    let conn = Connection::open(db_path)?;
    return Ok(conn);
}
