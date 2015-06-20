#![feature(plugin)]

#![plugin(regex_macros)]
extern crate regex;

#[macro_use] 
extern crate log;
extern crate env_logger;

use std::io::{BufRead, BufReader, Result};
use std::fs::File;
use regex::Regex;
use std::str::from_utf8;

// static REGEX_START: Regex = regex!(r"^\s*");

static REGEX_START: Regex = regex!(r"^\s*(?:(?P<unused>$|//|/\*|#\[)|(?P<fn>(?:pub\s+)?(?:unsafe\s+)?fn)|(?P<use>use\s)|(?P<struct>(?:pub\s+)?(?:enum|struct)\s)|(?P<impl>impl))");
static REGEX_FN: Regex = regex!(r"(?:pub\s+)?(?:unsafe\s+)?fn\s+(\w+)(?:.*->\s*(\w+))?");
static REGEX_USE: Regex = regex!(r"use\s+((?:\w+::)*)\{?((?:\s*(?:\*|\w+)\s*,?)+)\}?\s*;");
static REGEX_STRUCT: Regex = regex!(r"(?:pub\s+)?(?:enum|struct)\s+(\w+)(?:\s*<(?:\s*'?\w+\s*,?)+>)?\s*(;|\(|\{)");

#[derive(Debug,Clone,PartialEq)]
struct Token {
	name: String,  	// match name
	pos: usize 		// position in the file
}

#[derive(Debug,Clone,PartialEq)]
enum Searcheable {
    Fn((Token, Token)),							// (name, return type)
    Impl(Token, Token, Vec<(Token, Token)>),	// trait, struct, fns
    StructEnum(Token),
    Use((Vec<String>, Vec<Token>))				// (path, uses)
}

struct SearchIter {
    file: BufReader<File>,
    pos: usize,
    skip: Option<(u8, u8)>,
    buf: String
}


impl SearchIter {

	fn open(path: &str) ->  Result<SearchIter> {
		let file = try!(File::open(path));
		Ok(SearchIter {
			pos: 0,
			file: BufReader::new(file),
			buf: String::new(),
			skip: None
		})
	}
	
	fn next_line(&mut self) -> bool {

		if let Some((start, end)) = self.skip {
			self.consume(start, end);
			self.skip = None;
		}

		match self.file.read_line(&mut self.buf) {
			Err(_) | Ok(0) => false,
			Ok(len) => {
				self.pos += len;
				true
			}
		}
	}

	fn extend_until(&mut self, byte: u8) -> bool {

		debug!("extend for byte: {}, pos: {}", byte, self.pos);
		if self.buf.as_bytes().iter().find(|&b| *b == byte).is_some() { return true; }

		let mut bytes = Vec::new();
		match self.file.read_until(byte, &mut bytes) {
			Err(_) | Ok(0) => {
				self.buf.clear();
				return false
			},
			Ok(len) => {
				self.pos += len;
				self.buf.push_str(from_utf8(&bytes[..len]).unwrap());
			}
		}
		true
	}

	// consumes the file until end byte is found
	fn consume(&mut self, start: u8, end: u8) {

		debug!("consuming: ({}, {})", start, end);
		let mut level = 1;
		while level > 0 {
			let mut buf = Vec::new();
			match self.file.read_until(end, &mut buf) {
				Err(_) => return,
				Ok(len) => {						
					self.pos += len;
					if start > 0 { level += buf.iter().filter(|&&b| b == start).count(); }
					level -= 1;
					buf.clear();
				}
			}
		}
		debug!("consumed, pos is now at {}", self.pos);
	}

	fn match_fn(&mut self) -> Option<Searcheable> {

		if !self.extend_until(b'{') { return None; }

		debug!("extended pos: {}", self.pos);
		let m = if let Some(caps) = REGEX_FN.captures(&self.buf) {
			
			// name
			let (start, end) = caps.pos(1).unwrap();
			let mut pos = self.pos - self.buf.len();
			pos += start;

			let name = Token {
				name: self.buf[start..end].to_string(),
				pos: pos
			};

			let typ = match caps.pos(2) {
				Some((start, end)) => {					
					pos = self.pos - self.buf.len();
					pos += start;
					Token {
						name: self.buf[start..end].to_string(),
						pos: pos				
					}
				}, 
				None => {
					Token {
						name: String::new(),
						pos: pos				
					}
				}
			};
			Some(Searcheable::Fn((name, typ)))
		} else {
			None
		};
		self.buf.clear();
		self.skip = Some((b'{', b'}'));
		m
	}

	fn match_use(&mut self) -> Option<Searcheable> {

		if !self.extend_until(b';') { return None; }

		let m = if let Some(caps) = REGEX_USE.captures(&self.buf) {
			
			// members
			let members_str = caps.at(1).unwrap();

			let members = if members_str.len() > 2 { 
				members_str[..members_str.len()-2].split("::")
				.map(|s| s.to_string()).collect::<Vec<_>>()
			} else {
				Vec::new()
			};

			debug!("members: {:?}", members);

			let (start, end) = caps.pos(2).unwrap();
			let mut pos = self.pos - self.buf.len();
			pos += start;

			// set the output
			Some(Searcheable::Use((members, 
					self.buf[start..end].split(",").map(|s| Token{
					name:s.trim().to_string(),
					pos: pos
				}).collect())))
		} else {
			None
		};
		self.buf.clear();
		m
	}

	fn match_struct_or_enum(&mut self) -> Option<Searcheable> {

		debug!("struct");
		// append lines until there is a match
		// we don't know if it ends with ; or {}
		while !REGEX_STRUCT.is_match(&self.buf) {
			if !self.next_line() { return None; }
		}

		debug!("buf struct: {}", self.buf);
		let m = {
			let caps = REGEX_STRUCT.captures(&self.buf).unwrap();
			
			// found a match !
			let (start, end) = caps.pos(1).unwrap();
			let pos = self.pos - self.buf.len() + start;

			match caps.at(2) {
				Some("{") => {					
					debug!("struct has a {{: {}", self.buf);
					if !self.buf.contains('}') { self.skip = Some((b'{', b'}')); }
				}
				// Some("(") => {					
				// 	debug!("struct has a (");
				// 	if !self.buf.contains(';') {self.skip = Some((0, b';')); }
				// }
				_ => {}
			}

			// set the output
			Some(Searcheable::StructEnum(Token{
				name: self.buf[start..end].to_string(),
				pos: pos
			}))
		};

		self.buf.clear();
		m
	}

}

impl Iterator for SearchIter {
	type Item = Searcheable;

	fn next(&mut self) -> Option<Searcheable> {

		loop {
			if !self.next_line() { return None; }

			if let Some(caps) = REGEX_START.captures(&self.buf.clone()) {
				if let Some((name, _)) = caps.iter_named().find(|&(_, it)| it.is_some()) {
					match name {
						"use" 		=> return self.match_use(),
						"struct" 	=> return self.match_struct_or_enum(),
						"impl" 		=> debug!("impl"),
						"fn" 		=> return self.match_fn(),
						"unused" 	=> debug!("unused"),
						_ 			=> debug!("{:?}", name)
					}
				}
			}
			self.buf.clear();
		}
	}
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
				if let Searcheable::Fn((ref name, _)) = **m {
					if name.name == args[1] { return true; }
				}
				false
			});
		    println!("match: {:?}", m);
    	},
    	_ => println!("too many arguments")
    }

}
