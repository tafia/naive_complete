#![feature(plugin, io, path_ext)]
#![plugin(regex_macros)]
extern crate regex;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate toml;

mod file_parser;
mod func_parser;
mod file_searcher;

use func_parser::{FnParser, Scope};
use file_parser::{SearchIter, Searcheable};

#[derive(Debug,Clone,PartialEq)]
pub struct Token {
    pub name: String,      // match name
    pub pos: usize         // position in the file
}

fn main() {

    env_logger::init().unwrap();

    let args = std::env::args().collect::<Vec<_>>();

    if args.len() == 1 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    match &*args[1] {
        "prefix" => prefix(&args),
        // "complete" => complete(&match_fn),
        // // "complete-with-snippet" => complete(&match_with_snippet_fn),
        "find-definition" => find_definition(&args),
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
    println!("usage: {} complete linenum charnum fname", program);
    println!("or:    {} find-definition pos fname", program);
    println!("or:    {} complete fullyqualifiedname   (e.g. std::io::)", program);
    println!("or:    {} prefix pos fname", program);
    println!("or replace complete with complete-with-snippet for more detailed completions.");
}

// prefix pos fname
fn prefix(args: &Vec<String>) {
	 if args.len() != 4 {
        println!("Cannot run 'prefix', expect 3 arguments, found {}", args.len());
        print_usage(&args[0]);
    } else {
    	let pos = args[2].parse::<usize>().unwrap();
        let parser = FnParser::new(&args[3], 0, pos).unwrap();
        let name = match parser.scope() {
            Scope::Path(segments) => {
                if segments.len() == 0 {
                    println!("cannot find scope!");
                    return;
                }
                segments[0].clone()
            }
            Scope::Fn(segments) => {
                if segments.len() == 0 {
                    println!("cannot find scope!");
                    return;
                }
                segments[0].clone()
            }
            Scope::Word(word) => word,
        };
        println!("scope: {:?}", name);
        for it in parser.iter(&name.name, name.pos) {
            println!("fn item {:?}", it);
        }
    }
}

// find-definition pos fname
fn find_definition(args: &Vec<String>) {

    if args.len() != 4 {
        print_usage(&args[0]);
        return;
    }

    let pos = args[2].parse::<usize>().unwrap();
        // .expect(&format!("Cannot parse {} as usize", &args[2]));
    let file = &args[3];

    let iter = SearchIter::open(file).unwrap();
        // .expect(&format!("Cannot open file {}", file));

    let mut entries = iter.into_iter().take_while(|s|{
        debug!("entries until pos: {:#?}", s);
        match *s {
            Searcheable::Fn(_, Token {pos: p, ..})      |
            Searcheable::Impl(_, Token {pos: p, ..}, _) |
            Searcheable::StructEnum(Token {pos: p, ..}) |
            Searcheable::Const(_, Token {pos: p, ..})   |
            Searcheable::Trait(Token {pos: p, ..})      => p < pos,
            Searcheable::Use(_, ref v) => v.len() > 0 && v[0].pos < pos
        }
    }).collect::<Vec<_>>();

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
