#![feature(plugin)]
#![feature(io)]
#![plugin(regex_macros)]
extern crate regex;

#[macro_use]
extern crate log;
extern crate env_logger;

mod file_parser;
mod func_parser;

use file_parser::{SearchIter, Searcheable};
use func_parser::{SearchFnIter, Request};

#[derive(Debug,Clone,PartialEq)]
pub struct Token {
    pub name: String,      // match name
    pub pos: usize         // position in the file
}

#[cfg(not(test))]
fn print_usage() {
    let program = &std::env::args().next().unwrap();
    println!("usage: {} complete linenum charnum fname", program);
    println!("or:    {} find-definition pos fname", program);
    println!("or:    {} complete fullyqualifiedname   (e.g. std::io::)", program);
    println!("or:    {} prefix pos fname", program);
    println!("or replace complete with complete-with-snippet for more detailed completions.");
}

fn main() {

    env_logger::init().unwrap();

    let args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.len() == 0 {
        print_usage();
        std::process::exit(1);
    }

    match &*args[0] {
        "prefix" => {
        	if args.len() != 3 {        		
	            println!("Wrong number of arguments for {}", args[0]);
	            print_usage();
        	} else {
        		let iter = SearchFnIter::open(&args[2], 0, 
        			args[1].parse().unwrap()).unwrap();
        		println!("request: {:?}", iter.request());
        		for it in iter {
        			println!("fn item {:?}", it);
        		}
        	}
        },
        // "complete" => complete(&match_fn),
        // // "complete-with-snippet" => complete(&match_with_snippet_fn),
        // "find-definition" => find_definition(),
        // "help" => print_usage(),
        cmd => {
            println!("Sorry, I didn't understand command {}", cmd);
            print_usage();
            std::process::exit(1);
        }
    }

    // match args.len() {
    //     0 => print_usage(),
    //     1 => {
    //         let iter = SearchIter::open(&args[0]).unwrap();
    //         for v in iter {
    //             println!("match: {:?}", v);
    //         }
    //     },
    //     2 => {
    //         let mut iter = SearchIter::open(&args[0]).unwrap();
    //         let m = iter.find(|ref m| {
    //             if let Searcheable::Fn(ref name, _) = **m {
    //                 if name.name == args[1] { return true; }
    //             }
    //             false
    //         });
    //         println!("match: {:?}", m);
    //     },
    //     _ => println!("too many arguments")
    // }
}
