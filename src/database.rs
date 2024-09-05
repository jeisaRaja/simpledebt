use rusqlite::{params, Connection, Error, Result};

#[derive(Debug)]
pub struct User {
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

    pub fn create_person(&self, name: &String, balance: u64) {
        let _ = self.conn.execute(
            "INSERT INTO users (username, balance) VALUES (?1,?2)",
            params![name, balance],
        );
        print!("creating user");
    }

    pub fn update_person(&self, name: &String, update_balance: &u64) {
        let _ = self
            .conn
            .execute(
                "UPDATE users SET balance = balance + ?1 WHERE username = ?2 ",
                params![update_balance, name],
            )
            .expect("Failed to update user balance");
    }
}

fn connect_to_db() -> Result<Connection, Error> {
    let db_path = "utang.db";
    let conn = Connection::open(db_path)?;
    return Ok(conn);
}
