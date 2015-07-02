use std::io::{Read, Error};
use std::fs::File;
use regex::Regex;

use super::Token;

static REGEX_DECL: Regex = regex!(r"(?:let\s|\()\s*(\w+)(?:\s*:\s*(\w+))?");

#[derive(Debug,Clone,PartialEq)]
pub enum Request {
	Path(Vec<Token>),
	Fn(Vec<Token>),
	Word(Token)
}

pub struct SearchFnIter {
    pos: usize,
    buf: String
}

impl SearchFnIter {

    pub fn open(path: &str, offset: usize, pos: usize) ->  Result<SearchFnIter, Error> {
        debug!("creating SearchFnIter with on {}, {}", path, pos);
        let file = try!(File::open(path));
        let buf = file.chars().into_iter()
        	.skip(offset)
        	.map(|c| c.unwrap())
        	.enumerate()
		  	.take_while(|&(i, c)| 
		  		i < pos - offset || 
		  		c.is_alphabetic() || 
		  		c.is_numeric() ||
		  		c == '_')
		  	.map(|(_, c)| c).collect::<String>();
		
		debug!("buffer: {}", &buf);
		Ok(SearchFnIter {
            pos: offset,
            buf: buf
		})
    }

    pub fn request(&self) -> Request {

    	let ifn = self.buf.rfind('.').unwrap_or(0);
    	let ipath = self.buf.rfind(':').unwrap_or(0);
    	let iword = self.buf.rfind(|c: char| match c {
    		' ' | '\r' | '\n' | '\t' | '(' | '{' | '<' | '[' => true,
    		_ => false
    	}).unwrap_or(0);

		if ifn > ipath {
			if ifn > iword {
				Request::Fn(self.buf[iword..].split('.')
					.map(|s| Token { name: s.to_string(), pos: self.pos + ifn })
					.collect())
			} else {
				Request::Word(Token { name: self.buf[iword..].to_string(), pos: self.pos + iword })
			}
		} else {
			if ipath > iword {
				Request::Path(self.buf[iword..].split("::")
					.map(|s| Token { name: s.to_string(), pos: self.pos + ipath })
					.collect())
			} else {
				Request::Word(Token { name: self.buf[iword..].to_string(), pos: self.pos + iword })
			}
		}
    }
}

impl Iterator for SearchFnIter {
    type Item = (Token, Token); // name, type

    fn next(&mut self) -> Option<(Token, Token)> {

        if let Some(caps) = REGEX_DECL.captures(&self.buf[self.pos..]) {

            let (start, end) = caps.pos(1).unwrap();
            let name = Token { 
            	name: self.buf[self.pos+start..self.pos+end].to_string(), 
            	pos: self.pos+start 
            };

            let typ = if let Some((s, e)) = caps.pos(2) {
            	let pos = self.pos;
            	self.pos += e;
				Token { 
            		name: self.buf[pos+s..pos+e].to_string(), 
            		pos: pos+s
            	}
            } else {
            	self.pos += end;
            	Token { name: String::new(), pos: self.pos }
            };

            Some((name, typ))
        } else {
        	None
        }
    }
}
