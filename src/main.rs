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
    pub name: String,   // match name
    pub pos: usize      // position in the file
}

fn main() {

    env_logger::init().unwrap();

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    match &*args[1] {
        "find-definition" => {
            let (pos, file) = parse_pos_and_file(&args);
            find_definition(&file, pos)
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

fn parse_pos_and_file(args: &Vec<String>) -> Option<(usize, String)> {
    if args.len() != 4 {
        print_usage(&args[0]);
        return None;
    }
    let file = &args[3];
    args[2].parse::<usize>().ok().map(|pos| (pos, file))
}

// find-definition pos fname
fn find_definition(file: &str, pos: usize) {

    // search for all file entries up to requested `pos`, and save the offset
    let iter = SearchIter::open(file).unwrap();
    
    // get the scope search (Searcheable item)
    let mut offset = 0;
    let searcheable = iter.cloned().find(|s| {
        debug!("entries until pos: {:#?}", s);
        let end = match *s {
            Searcheable::Fn(_, Token {pos: p, ..})      |
            Searcheable::Impl(_, Token {pos: p, ..}, _) |
            Searcheable::StructEnum(Token {pos: p, ..}) |
            Searcheable::Const(_, Token {pos: p, ..})   |
            Searcheable::Trait(Token {pos: p, ..})      => p,
            Searcheable::Use(_, ref v) => v.len() > 0 { v[0].pos } else { 0 }
        }
        if end > pos { return false; }
        offset = end;
        true
    });
    
    // get the fn parser for the Searcheable item
    let mut innerScope = FnParser::new(file, offset, pos);
    debug!("root searchable:\n{:#?}", innerScope);
    
    let scope = innerScope.scope();
    debug!("root scope:\n{:?}", scope);
    
    let first_word = match scope {
        Scope::Path(segments) |
        Scope::Fn(segments)   => {
            if segments.len() == 0 {
                println!("cannot find scope!");
                return;
            }
            segments[0]
        }
        Scope::Word(word) => word
    }.clone();
    
    search_word(&first_word, &innerScope.iter(first_word, pos), &searcheable);
    
}

fn search_word(word: &str, iter: mut FnIter, searcheable: &Option<Searcheable>
    search_iter: &mut SearchIter) -> Option<Token> 
{
    iter.next().or(search_iter.find(|&s| match *s {
        Searcheable::Fn(Token {name: name, ..}, _)      |
        Searcheable::StructEnum(Token {name: name, ..}) |
        Searcheable::Const(Token {name: name, ..}, _)   => name.starts_with(word),
        _ => false
    })
}
