use chrono::{DateTime, Local};
use dirs::home_dir;
use rusqlite::{params, Connection, Error, Result};
use std::fs;

#[derive(Debug)]
pub struct User {
    id: i32,
    username: String,
    balance: i64,
}

#[derive(Debug)]
pub struct Transaction {
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
        description: &String,
    ) {
        let _user = self.conn.execute(
            "INSERT INTO users (username, balance) VALUES (?1,?2) RETURNING id",
            params![name, balance],
        );
        if balance != 0 {
            println!("balance is {}, inserting to transactions", balance);
            let user_id = self.conn.last_insert_rowid();
            println!("{}", user_id);
            let amount: i64 = match transaction_type.as_str() {
                "pay" | "lend" => balance as i64,
                "receive" | "borrow" => -(balance as i64),
                _ => balance.try_into().unwrap(),
            };
            let date_now = Local::now().to_string();
            let _ = self.conn.execute(
                "INSERT INTO transactions (user_id, transaction_type, amount, date, description) VALUES (?1,?2,?3,?4,?5)",
                params![user_id, transaction_type, amount, date_now, description],
            ).unwrap();
        };
        print!("creating user\n");
    }

    pub fn give_to(
        &self,
        name: &String,
        balance: &u64,
        transaction_type: String,
        description: &String,
    ) {
        let user_id = self
            .conn
            .query_row(
                "SELECT id FROM users WHERE username = ?1",
                params![name],
                |row| row.get(0),
            )
            .expect("Failed to fetch user id");
        let _give = self
            .conn
            .execute(
                "UPDATE users SET balance = balance + ?1 WHERE username = ?2 ",
                params![balance, name],
            )
            .expect("Failed to update user balance");
        self.insert_transaction(user_id, balance, transaction_type, description)
    }

    pub fn receive_from(
        &self,
        name: &String,
        balance: &u64,
        transaction_type: String,
        description: &String,
    ) {
        let user_id: i32 = self
            .conn
            .query_row(
                "SELECT id FROM users WHERE username = ?1",
                params![name],
                |row| row.get(0),
            )
            .expect("Failed to fetch user id");
        let _ = self
            .conn
            .execute(
                "UPDATE users SET balance = balance - ?1 WHERE username = ?2 ",
                params![balance, name],
            )
            .expect("Failed to update user balance");
        self.insert_transaction(user_id, balance, transaction_type, description)
    }

    fn insert_transaction(
        &self,
        user_id: i32,
        balance: &u64,
        transaction_type: String,
        description: &String,
    ) {
        let date_now = Local::now().to_string();
        let _ = self.conn.execute(
            "INSERT INTO transactions 
        (user_id, transaction_type, amount, date, description) 
     VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, transaction_type, balance, date_now, description],
        );
    }

    pub fn last_transactions(&self, count: u8) -> Result<Vec<Transaction>, rusqlite::Error> {
        let mut transactions_vec: Vec<Transaction> = vec![];
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, transaction_type, amount, date, description FROM transactions ORDER BY ID DESC LIMIT ?1;",
        ).unwrap();
        let result = stmt.query_map([&count], |row| {
            let date_str: String = row.get(4)?;
            let date: DateTime<Local> = date_str.parse::<DateTime<Local>>().map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(Transaction {
                id: row.get(0).unwrap(),
                user_id: row.get(1).unwrap(),
                transaction_type: row.get(2).unwrap(),
                amount: row.get(3).unwrap(),
                date,
                description: row.get(5).unwrap_or("description".to_string()),
            })
        });

        for transaction in result? {
            transactions_vec.push(transaction?)
        }
        Ok(transactions_vec)
    }
}

fn connect_to_db() -> Result<Connection, Error> {
    if let Some(home_path) = home_dir() {
        let db_dir = home_path.join(".local").join("share").join("utang");
        let db_path = db_dir.join("utang.db");

        if !db_dir.exists() {
            fs::create_dir_all(&db_dir).expect("Failed to create database directory")
        }
        if !db_path.exists() {
            let conn = Connection::open(&db_path).unwrap();
            let _ = conn.execute(
                "CREATE TABLE users (
                id integer primary key autoincrement,
                username text not null,
                balance integer default 0,
                UNIQUE(username)
                );
                CREATE TABLE transactions (
                id integer primary key autoincrement,
                user_id integer not null,
                transaction_type text not null,
                amount integer not null,
                date text not null,
                description text,
                foreign key(user_id) references users(id)
                );
                ",
                [],
            );
        } else {
            println!("database found at: {:?}", db_path);
        }
        let conn = Connection::open(db_path)?;
        return Ok(conn);
    } else {
        return Err(rusqlite::Error::InvalidQuery);
    }
}
