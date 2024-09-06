use clap::{Parser, ValueEnum};
use database::DB;
use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};
use std::io::Write;
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

    let is_name = check_argument(cli.name.as_ref());
    let is_amount = check_argument(cli.amount.as_ref());

    match cli.command {
        Command::Pay => {
            if !(is_name && is_amount) {
                println!("Provide name and amount");
                return;
            }
            let name = cli.name.as_ref().expect("name should be a String");
            let amount = cli.amount.as_ref().expect("amount should be a u64");
            let person = db.select_person(name);
            if person.is_err() {
                ask_user_to_create_person(&db, name, amount);
                return;
            }
            db.give_to(name, amount);
            let amount_separator = cli.amount.unwrap().to_formatted_string(&Locale::en);
            println!("Paying {} Rp{}", cli.name.unwrap(), amount_separator);
        }

        Command::Lend => {
            if !(is_name && is_amount) {
                println!("Provide name and amount")
            }
            let name = cli.name.as_ref().expect("name should be a String");
            let amount = cli.amount.as_ref().expect("amount should be a u64");
            let person = db.select_person(name);
            if person.is_err() {
                ask_user_to_create_person(&db, name, amount);
                return;
            }
            db.receive_from(name, amount);
            let amount_separator = cli.amount.unwrap().to_formatted_string(&Locale::en);
            print!("Lending {} Rp.{}", cli.name.unwrap(), amount_separator)
        }

        Command::Check => {
            println!("Checking");
            if !is_name {
                println!("Provide name")
            }
            let s = db.select_person(cli.name.as_ref().expect("name should be a String"));
            if s.is_err() {
                println!("Error when trying to get name: {:?}", cli.name);
                return;
            }
            println!("{:?}", s);
        }

        Command::Borrow => {
            println!("Borrowing")
        }

        Command::Receive => {
            println!("Receiving")
        }
    }
}

fn check_argument<T>(field: Option<&T>) -> bool {
    field.is_some()
}

fn ask_user_to_create_person(db: &DB, name: &String, amount: &u64) {
    println!("This person is not in the database, do you want to add them? y/n ");
    print!(">> ");
    std::io::stdout().flush().unwrap();
    let mut ans = String::new();
    let _tmp = std::io::stdin().read_line(&mut ans).unwrap();
    ans = ans.trim().to_string();
    if ans.eq_ignore_ascii_case("y") {
        db.create_person(name, *amount)
    }
    return;
}
