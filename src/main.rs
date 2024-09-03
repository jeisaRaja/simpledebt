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
    name: Option<String>,
    amount: Option<u64>,
}

fn main() {
    let cli = Cli::parse();
    let db = database::DB::new();

    match cli.command {
        Command::Pay => {
            if cli.name.is_some() && cli.amount.is_some() {
                println!("Paying {:?} Rp. {:?}", cli.name, cli.amount)
            }
        }
        Command::Lend => {
            println!("Lending")
        }
        Command::Check => {
            println!("Checking");
            if cli.name.is_some() {
                let s = db.select_person(cli.name.expect("name should be a string"));
                println!("{:?}", s);
            }
        }
        Command::Borrow => {
            println!("Borrowing")
        }
        Command::Receive => {
            println!("Receiving")
        }
    }
}
