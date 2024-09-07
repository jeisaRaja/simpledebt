use clap::{Parser, ValueEnum};
use database::DB;
use num_format::{Locale, ToFormattedString};
use std::io::Write;
mod database;

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
                ask_user_to_create_person(&db, name, amount, Command::Pay);
                return;
            }
            db.give_to(name, amount, "pay".to_string(), None);
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
                ask_user_to_create_person(&db, name, amount, Command::Lend);
                return;
            }
            db.receive_from(name, amount, "lend".to_string(), None);
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
            let name = cli.name.as_ref().expect("name should be a String");
            let amount = cli.amount.as_ref().expect("amount should be a u64");
            let person = db.select_person(name);
            if person.is_err() {
                ask_user_to_create_person(&db, name, amount, Command::Borrow);
                return;
            }
            db.receive_from(name, amount, "borrow".to_string(), None);
            let amount_separator = cli.amount.unwrap().to_formatted_string(&Locale::en);
            println!("Borrowing {} Rp{}", cli.name.unwrap(), amount_separator);
        }

        Command::Receive => {
            let name = cli.name.as_ref().expect("name should be a String");
            let amount = cli.amount.as_ref().expect("amount should be a u64");
            let person = db.select_person(name);
            if person.is_err() {
                ask_user_to_create_person(&db, name, amount, Command::Receive);
                return;
            }
            db.receive_from(name, amount, "receiving".to_string(), None);
            let amount_separator = cli.amount.unwrap().to_formatted_string(&Locale::en);
            println!("Receiving {} Rp{}", cli.name.unwrap(), amount_separator);
        }
    }
}

fn check_argument<T>(field: Option<&T>) -> bool {
    field.is_some()
}

fn ask_user_to_create_person(db: &DB, name: &String, amount: &u64, cmd_type: Command) {
    println!("This person is not in the database, do you want to add them? y/n ");
    print!(">> ");
    std::io::stdout().flush().unwrap();
    let mut ans = String::new();
    let _tmp = std::io::stdin().read_line(&mut ans).unwrap();
    ans = ans.trim().to_string();
    if ans.eq_ignore_ascii_case("y") {
        let cmd_type = match cmd_type {
            Command::Pay => "pay".to_string(),
            Command::Receive => "receive".to_string(),
            Command::Lend => "lend".to_string(),
            Command::Borrow => "borrow".to_string(),
            Command::Check => panic!("Check command cannot create user!"),
        };
        db.create_person(name, *amount, cmd_type, None)
    }
    return;
}
