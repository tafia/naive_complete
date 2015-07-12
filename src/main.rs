#![feature(plugin, io, path_ext, append)]
#![plugin(regex_macros)]
extern crate regex;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate toml;

mod file_parser;
mod func_parser;
mod file_searcher;
mod manager;

use manager::{find_definition};

fn main() {

    env_logger::init().unwrap();

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    match &*args[1] {
        "find-definition" => {
            if let Some((pos, file)) = parse_pos_and_file(&args) {
                if let Some(t) = find_definition(&file, pos) {
                    println!("Found defition: {:#?}", t);
                }
            }
        },
        "help" => print_usage(&args[0]),
        cmd => {
            println!("Sorry, I didn't understand command {}", cmd);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

#[cfg(not(test))]
fn print_usage(program: &str) {
    println!("usage: {} complete pos fname", program);
    println!("or:    {} find-definition pos fname", program);
    println!("or:    {} complete fullyqualifiedname   (e.g. std::io::)", program);
    println!("or:    {} prefix pos fname", program);
    println!("or replace complete with complete-with-snippet for more detailed completions.");
}

fn parse_pos_and_file(args: &Vec<String>) -> Option<(usize, &str)> {
    if args.len() != 4 {
        print_usage(&args[0]);
        return None;
    }
    let file = &*args[3];
    args[2].parse::<usize>().ok().map(|pos| (pos, file))
}
