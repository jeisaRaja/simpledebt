use rusqlite::{Connection, Error, Result};

struct DB {
    conn: Connection,
}

impl DB {
    fn new() -> DB {
        let conn = connect_to_db().unwrap();
        return DB { conn };
    }
}

fn connect_to_db() -> Result<Connection, Error> {
    let db_path = "utang.db";
    let conn = Connection::open(db_path)?;
    //conn.execute(
    //    "INSERT INTO users (username, balance) VALUES (?1, ?2)",
    //    (&user.username, &user.balance),
    //)?;

    return Ok(conn);
}
