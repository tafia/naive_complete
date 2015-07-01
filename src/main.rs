#![feature(plugin)]

#![plugin(regex_macros)]
extern crate regex;

#[macro_use]
extern crate log;
extern crate env_logger;

mod search_function;
mod search_file;

use search_file::SearchIter;
use search_file::Searcheable;

#[derive(Debug,Clone,PartialEq)]
pub struct Token {
    pub name: String,      // match name
    pub pos: usize         // position in the file
}

fn main() {

    env_logger::init().unwrap();

    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.len() {
        0 => println!("no argument"),
        1 => {
            let iter = SearchIter::open(&args[0]).unwrap();
            for v in iter {
                println!("match: {:?}", v);
            }
        },
        2 => {
            let mut iter = SearchIter::open(&args[0]).unwrap();
            let m = iter.find(|ref m| {
                if let Searcheable::Fn(ref name, _) = **m {
                    if name.name == args[1] { return true; }
                }
                false
            });
            println!("match: {:?}", m);
        },
        _ => println!("too many arguments")
    }
}
