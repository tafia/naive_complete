use std::io::{Read, Error, ErrorKind};
use std::fs::File;
use regex::Regex;
use std::str::from_utf8;

use super::Token;

static REGEX_LET: Regex = regex!(r"(?:if\s+)?let\s+(\w+)\s+=");

#[derive(Debug,Clone,PartialEq)]
pub enum FnSearcheable {
    Let(Token),                            // (name, return type)
    Match(Token, Token)    // trait, struct, fns    
}

struct SearchFnIter {
    pos: usize,
    skip: Option<(u8, u8)>,
    buf: Vec<u8>
}

impl SearchFnIter {

    pub fn open(path: &str, offset: usize, pos: usize) ->  Result<SearchFnIter, Error> {
        let file = try!(File::open(path));
        let mut bytes = file.bytes().skip(offset);
        let buf = bytes.map(|r| r.unwrap())
    		  	  .take(pos - offset).take_while(|&b|
        	match b {
        		b'a'...b'z' | 
        		b'A'...b'Z' |  
        		b'0'...b'9' | 
        		b'_' => true,
        		_ => false,
        	}).collect();
		
		Ok(SearchFnIter {
            pos: offset,
            buf: buf,
	        skip: None
		})
    }
}

impl Iterator for SearchFnIter {
    type Item = FnSearcheable;

    fn next(&mut self) -> Option<FnSearcheable> {

    	None
        // loop {

        //     if let Some(caps) = REGEX_START.captures(&self.buf.clone()) {
        //         if let Some((name, _)) = caps.iter_named().find(|&(_, it)| it.is_some()) {
        //             match name {
        //                 "use"         => return self.match_use(),
        //                 "struct"     => return self.match_struct_or_enum(),
        //                 "impl"         => return self.match_impl(),
        //                 "fn"         => return self.match_fn(),
        //                 "const"        => return self.match_const(),
        //                 "trait"        => return self.match_trait(),
        //                 "unused"     => debug!("unused"),
        //                 _             => debug!("{:?}", name)
        //             }
        //         }
        //     }
        //     self.buf.clear();
        // }
    }
}
