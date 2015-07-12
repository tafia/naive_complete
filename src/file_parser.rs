use std::io::{BufRead, BufReader, Result};
use std::fs::File;
use std::str::from_utf8;

use regex::Regex;

use manager::Token;

static REGEX_START: Regex = regex!("^\\s*(?:\
                                    (?P<unused>$|//|/\\*|#\\[)|\
                                    (?P<fn>(?:pub\\s+)?(?:unsafe\\s+)?fn)|\
                                    (?P<use>use\\s)|\
                                    (?P<struct>(?:pub\\s+)?(?:enum|struct)\\s)|\
                                    (?P<impl>impl)|\
                                    (?P<const>(?:pub\\s+)?(?:const|static))|\
                                    (?P<trait>(?:pub\\s+)?trait)\
                                    )");
static REGEX_FN: Regex = regex!(r"(?:pub\s+)?(?:unsafe\s+)?fn\s+(\w+)(?:.*->\s*(\w+))?");
static REGEX_USE: Regex = regex!(r"use\s+((?:\w+::)*)\{?((?:\s*(?:\*|\w+)\s*,?)+)\}?\s*;");
static REGEX_STRUCT: Regex = regex!(r"(?:pub\s+)?(?:enum|struct)\s+(\w+).*(;|\{)");
static REGEX_IMPL: Regex = regex!(r"impl(?:\s*<.*>)?\s+(?:(\w+).*\sfor\s+)?(&?\w+)");
static REGEX_CONST: Regex = regex!(r"(?:pub\s+)?(?:static|const)\s+(\w+)\s*:.*(\w+)");
static REGEX_TRAIT: Regex = regex!(r"(?:pub\s+)?trait\s+(\w+)");


#[derive(Debug,Clone,PartialEq)]
pub enum Searcheable {
    Fn(Token, Token),                            // (name, return type)
    Impl(Token, Token, Vec<(Token, Token)>),    // trait, struct, fns
    StructEnum(Token),
    Use(Vec<String>, Vec<Token>),                // (path, uses)
    Const(Token, Token),                        // name, type
    Trait(Token)                                // name
}

pub struct SearchIter {
    file: BufReader<File>,
    pos: usize,
    skip: Option<(u8, u8)>,
    buf: String
}

impl SearchIter {

    pub fn open(path: &str) ->  Result<SearchIter> {
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
        let mut level = 1;
        while level > 0 {
            let mut buf = Vec::new();
            match self.file.read_until(end, &mut buf) {
                Err(_) => return,
                Ok(len) => {
                    self.pos += len;
                    if start > 0 { level += buf.iter().filter(|&&b| b == start).count(); }
                    level -= 1;
                }
            }
        }
        debug!("consumed until {}", self.pos);
    }

    fn match_fn(&mut self) -> Option<Searcheable> {

        if !self.extend_until(b'{') { return None; }

        debug!("extended pos: {}", self.pos);
        let m = if let Some(caps) = REGEX_FN.captures(&self.buf) {

            let (start, end) = caps.pos(1).unwrap();

            let buf_start = self.pos - self.buf.len();
            let name = Token {
                name: self.buf[start..end].to_string(),
                pos: buf_start + start
            };

            let typ = match caps.pos(2) {
                Some((start, end)) => {
                    Token {
                        name: self.buf[start..end].to_string(),
                        pos: buf_start + start
                    }
                },
                None => Token {
                        name: String::new(),
                        pos: buf_start + start
                    }
            };
            Some(Searcheable::Fn(name, typ))
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
            let members_str = caps.at(1).unwrap();
            let members = if members_str.len() > 2 {
                members_str[..members_str.len()-2].split("::")
                .map(|s| s.to_string()).collect::<Vec<_>>()
            } else { Vec::new() };
            debug!("members: {:?}", members);

            let (start, end) = caps.pos(2).unwrap();
            let mut pos = self.pos - self.buf.len();
            pos += start;
            Some(Searcheable::Use(members,
                    self.buf[start..end].split(",").map(|s| Token{
                    name:s.trim().to_string(),
                    pos: pos
                }).collect()))
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
        let m = if let Some(caps) = REGEX_STRUCT.captures(&self.buf) {
            let (start, end) = caps.pos(1).unwrap();
            let pos = self.pos - self.buf.len() + start;
            match caps.at(2) {
                Some("{") => {
                    if !self.buf.contains('}') { self.skip = Some((b'{', b'}')); }
                }
                _ => {} // TODO: manage each alternatives
            }
            Some(Searcheable::StructEnum(Token{
                name: self.buf[start..end].to_string(),
                pos: pos
            }))
        } else {
            None
        };

        self.buf.clear();
        m
    }

    fn match_impl(&mut self) -> Option<Searcheable> {

        debug!("match impl, pos: {}, buf: {}", self.pos, self.buf);
        let m = if let Some(caps) = REGEX_IMPL.captures(&self.buf) {

            let buf_start = self.pos - self.buf.len();

            let trait_part = match caps.pos(1) {
                Some((start, end)) => {
                    Token {
                        name: self.buf[start..end].to_string(),
                        pos: buf_start + start
                    }
                },
                None => Token {
                        name: String::new(),
                        pos: buf_start
                    }
            };

            let (start, end) = caps.pos(2).unwrap();
            let struct_part = Token {
                name: self.buf[start..end].to_string(),
                pos: buf_start + start
            };

            Some(Searcheable::Impl(trait_part, struct_part, Vec::new()))
        } else {
            None
        };
        self.buf.clear();
        self.skip = Some((b'{', b'}'));
        m
    }

    fn match_const(&mut self) -> Option<Searcheable> {

        if !self.extend_until(b';') { return None; }

        let m = if let Some(caps) = REGEX_CONST.captures(&self.buf) {
            let buf_start = self.pos - self.buf.len();

            let (start, end) = caps.pos(1).unwrap();
            let const_name = Token {
                name: self.buf[start..end].to_string(),
                pos: buf_start + start
            };

            let (start, end) = caps.pos(2).unwrap();
            let const_type = Token {
                name: self.buf[start..end].to_string(),
                pos: buf_start + start
            };

            Some(Searcheable::Const(const_name, const_type))
        } else {
            None
        };

        self.buf.clear();
        m
    }

    fn match_trait(&mut self) -> Option<Searcheable> {

        let m = if let Some(caps) = REGEX_TRAIT.captures(&self.buf) {

            let (start, end) = caps.pos(1).unwrap();

            let buf_start = self.pos - self.buf.len();
            let name = Token {
                name: self.buf[start..end].to_string(),
                pos: buf_start + start
            };
            Some(Searcheable::Trait(name))
        } else {
            None
        };
        self.buf.clear();
        self.skip = Some((b'{', b'}'));
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
                        "use"    => return self.match_use(),
                        "struct" => return self.match_struct_or_enum(),
                        "impl"   => return self.match_impl(),
                        "fn"     => return self.match_fn(),
                        "const"  => return self.match_const(),
                        "trait"  => return self.match_trait(),
                        "unused" => debug!("unused ({})", self.pos),
                        _        => debug!("{:?}", name)
                    }
                }
            }
            self.buf.clear();
        }
    }
}
