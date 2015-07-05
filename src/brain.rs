use file_parser;
use func_parser::{FnParser, Scope};
use super::print_usage;


pub fn prefix(args: &Vec<String>) {
	 if args.len() != 3 {        		
        println!("Cannot run 'prefix', expect 3 arguments, found {}", args.len());
        print_usage();
    } else {
    	let pos = args[1].parse::<usize>().unwrap();
        let parser = FnParser::new(&args[2], 0, pos).unwrap();
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
