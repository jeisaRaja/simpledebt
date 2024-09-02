use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Loan {
    lender: String,
    borrower: String,
    amount: u64,
    description: String,
}

struct User {
    id: i32,
    username: String,
    balance: u64,
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

#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}

fn main() -> Result<()> {
    let db_path = "utang.db";

    let conn = Connection::open(db_path)?;

    let user = User::new("frendo".to_string());

    conn.execute(
        "INSERT INTO users (username, balance) VALUES (?1, ?2)",
        (&user.username, &user.balance),
    )?;

    Ok(())
}
