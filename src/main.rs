use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
mod database;

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

#[derive(Debug, ValueEnum, Clone, Copy)]
enum Command {
    Pay,
    Receive,
    Borrow,
    Lend,
    Check,
}

#[derive(Parser, Debug)]
struct Cli {
    #[arg(value_enum)]
    command: Command,
    name: String,
    amount: u64,
}

fn main() {
    let cli = Cli::parse();

    println!("{:?}", cli)
}
