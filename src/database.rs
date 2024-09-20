use chrono::{DateTime, Local};
use core::fmt;
use dirs::home_dir;
use num_format::{Locale, ToFormattedString};
use rusqlite::{params, Connection, Error, Result};
use std::fs;

pub struct UserWithTransactions {
    pub user: User,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub balance: i64,
}

#[derive(Debug)]
pub struct Transaction {
    pub username: String,
    pub transaction_type: String,
    pub amount: i64,
    pub date: DateTime<Local>,
    pub description: String,
}

impl core::fmt::Display for UserWithTransactions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted_balance: String;
        if self.user.balance < 0 {
            formatted_balance = format!(
                "-Rp{}",
                (-self.user.balance).to_formatted_string(&Locale::en)
            );
        } else {
            formatted_balance = format!("Rp{}", self.user.balance.to_formatted_string(&Locale::en));
        }
        write!(
            f,
            "{:<13}: {}\n{:<13}: {}\n{:<13}:\n",
            "Name", self.user.username, "Balance", formatted_balance, "Transactions"
        )?;

        write!(
            f,
            "{:<13} {:<10} {:<13} {}\n",
            "Type", "Amount", "Date", "Description"
        )?;
        write!(f, "{}\n", "-".repeat(55))?;
        for (index, transaction) in self.transactions.iter().enumerate() {
            write!(f, "{}. {}", index + 1, transaction)?
        }
        Ok(())
    }
}

impl core::fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted_amount = self.amount.to_formatted_string(&Locale::en);
        write!(
            f,
            "{:<10} {:<10} {:<10} {:<13} {}\n",
            self.username,
            self.transaction_type,
            format!("Rp{}", formatted_amount),
            self.date.date_naive().to_string(),
            self.description
        )
    }
}

impl User {
    fn new(username: String) -> User {
        return User {
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
            .prepare("SELECT username, balance FROM users WHERE username = ?1")?;
        let user = stmt.query_row([&name], |row| {
            Ok(User {
                username: row.get(0)?,
                balance: row.get(1)?,
            })
        })?;
        Ok(user)
    }

    pub fn check_person(&self, name: &String) -> Result<UserWithTransactions, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT  users.username, users.balance, transactions.transaction_type, transactions.amount, 
        transactions.description, transactions.date FROM users INNER JOIN transactions ON users.id = transactions.user_id WHERE users.username = ?1 ORDER BY transactions.date DESC LIMIT 5")?;
        let user = stmt.query_row([&name], |row| {
            Ok(User {
                username: row.get(0)?,
                balance: row.get(1)?,
            })
        })?;
        let mut transactions = vec![];
        let transaction_rows = stmt.query_map([&name], |row| {
            let date_string: String = row.get(5)?;
            let date: DateTime<Local> = date_string.parse::<DateTime<Local>>().map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(Transaction {
                username: row.get(0)?,
                transaction_type: row.get(2)?,
                amount: row.get(3)?,
                description: row.get(4)?,
                date,
            })
        })?;
        for transaction in transaction_rows {
            transactions.push(transaction?);
        }

        Ok(UserWithTransactions { user, transactions })
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
            "SELECT  users.username, transactions.transaction_type, transactions.amount, 
        transactions.description, transactions.date FROM users INNER JOIN transactions ON users.id = transactions.user_id ORDER BY transactions.date DESC LIMIT ?1;",
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
                username: row.get(0)?,
                transaction_type: row.get(1)?,
                amount: row.get(2)?,
                date,
                description: row.get(3).unwrap_or("-".to_string()),
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
            let _ = conn.execute(
                "CREATE TABLE transactions (
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
