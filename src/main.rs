use clap::{Arg, Command as Clap_Command, ValueEnum};
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

fn main() {
    let matches = Clap_Command::new("utang")
        .version("1.0")
        .author("raja")
        .about("Cli tool for managing debt")
        .subcommand(
            Clap_Command::new("pay")
                .about("Pay a certain amount to a user")
                .arg(Arg::new("name").required(true).index(1))
                .arg(Arg::new("amount").required(true).index(2))
                .arg(Arg::new("description").index(3)),
        )
        .subcommand(
            Clap_Command::new("receive")
                .about("Receive a certain amount from a user")
                .arg(Arg::new("name").required(true).index(1))
                .arg(Arg::new("amount").required(true).index(2))
                .arg(Arg::new("description").index(3)),
        )
        .subcommand(
            Clap_Command::new("check")
                .about("Check balance or transactions info")
                .arg(Arg::new("name").index(1))
                .arg(Arg::new("range").index(2)),
        )
        .subcommand(
            Clap_Command::new("lend")
                .about("Lend a certain amount to a user")
                .arg(Arg::new("name").required(true).index(1))
                .arg(Arg::new("amount").required(true).index(2))
                .arg(Arg::new("description").index(3)),
        )
        .subcommand(
            Clap_Command::new("borrow")
                .about("Borrow a certain amount from a user")
                .arg(Arg::new("name").required(true).index(1))
                .arg(Arg::new("amount").required(true).index(2))
                .arg(Arg::new("description").index(3)),
        )
        .get_matches();
    let db = database::DB::new();
    let default_description = "not provided".to_string();

    match matches.subcommand() {
        Some(("pay", sub)) => {
            let name = sub.get_one::<String>("name").unwrap();
            let amount: u64 = sub
                .get_one::<String>("amount")
                .unwrap()
                .parse()
                .expect("failed to parse amount to u64");
            let description = sub
                .get_one::<String>("description")
                .unwrap_or(&default_description);
            let person = db.select_person(name);
            if person.is_err() {
                ask_user_to_create_person(&db, name, &amount, Command::Pay, description);
                return;
            }
            db.give_to(name, &amount, "pay".to_string(), description);
            let amount_separator = amount.to_formatted_string(&Locale::en);
            println!("Paying {} Rp{}", name, amount_separator);
        }
        Some(("receive", sub)) => {
            let name = sub.get_one::<String>("name").unwrap();
            let amount: u64 = sub
                .get_one::<String>("amount")
                .unwrap()
                .parse()
                .expect("failed to parse amount to u64");
            let description = sub
                .get_one::<String>("description")
                .unwrap_or(&default_description);
            let person = db.select_person(name);
            if person.is_err() {
                ask_user_to_create_person(&db, name, &amount, Command::Receive, description);
                return;
            }
            db.receive_from(name, &amount, "receive".to_string(), description);
            let amount_separator = amount.to_formatted_string(&Locale::en);
            println!("Receiving {} Rp{}", name, amount_separator);
        }
        Some(("lend", sub)) => {
            let name = sub.get_one::<String>("name").unwrap();
            let amount: u64 = sub
                .get_one::<String>("amount")
                .unwrap()
                .parse()
                .expect("failed to parse amount to u64");
            let description = sub
                .get_one::<String>("description")
                .unwrap_or(&default_description);
            let person = db.select_person(name);
            if person.is_err() {
                ask_user_to_create_person(&db, name, &amount, Command::Lend, description);
                return;
            }
            db.give_to(name, &amount, "lend".to_string(), description);
            let amount_separator = amount.to_formatted_string(&Locale::en);
            println!("Lending {} Rp{}", name, amount_separator);
        }
        Some(("borrow", sub)) => {
            let name = sub.get_one::<String>("name").unwrap();
            let amount: u64 = sub
                .get_one::<String>("amount")
                .unwrap()
                .parse()
                .expect("failed to parse amount to u64");
            let description = sub.get_one::<String>("description").unwrap();
            let person = db.select_person(name);
            if person.is_err() {
                ask_user_to_create_person(&db, name, &amount, Command::Receive, description);
                return;
            }
            db.receive_from(name, &amount, "borrow".to_string(), description);
            let amount_separator = amount.to_formatted_string(&Locale::en);
            println!("Receiving {} Rp{}", name, amount_separator);
        }
        Some(("check", sub)) => {
            let name = sub.get_one::<String>("name");
            let range: u8 = sub
                .get_one::<String>("range")
                .map(|r| r.parse::<u8>().expect("failed to parse range to u8"))
                .unwrap_or(5);
            if name.is_none() {
                let transactions = db.last_transactions(range).unwrap();
                for i in transactions {
                    println!("{:?}", i);
                }
                return;
            }
            let s = db.select_person(name.unwrap());
            if s.is_err() {
                println!("Error when trying to get name: {:?}", name);
                return;
            }
            println!("{:?}", s);
        }
        Some((_, _)) => {
            println!("none")
        }
        None => println!("none"),
    }
}

fn ask_user_to_create_person(
    db: &DB,
    name: &String,
    amount: &u64,
    cmd_type: Command,
    description: &String,
) {
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
        db.create_person(name, *amount, cmd_type, description)
    }
    return;
}
