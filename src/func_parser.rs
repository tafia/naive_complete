use std::io::{Read, Error};
use std::fs::File;

use regex::Regex;

use super::Token;

static REGEX_DEF: Regex = regex!("^\\s*(?:\
                                  (?P<let>(?:if\\s+)?let\\s+)|\
                                  (?P<use>use\\s)
                                  )");
static REGEX_ARG: Regex = regex!("(?:\
                                  (?P<fn>\\((\\s*(\\w+)\\s*:\\s*&?\\w+\\s*,?)+\\))|\
                                  (?P<closure>\\|(\\s*(\\w+)\\s*(?::\\s*&?\\w+\\s*),?)+\\|)\
                                  )");

#[derive(Debug,Clone,PartialEq)]
pub enum Scope {
    Path(Vec<Token>),
    Fn(Vec<Token>),
    Word(Token)
}

#[derive(Debug)]
pub struct FnParser {
    start: usize,
    buf: String
}

impl FnParser {

    pub fn new(path: &str, offset: usize, pos: usize) ->  Result<FnParser, Error> {
        debug!("creating FnParser file: {}, offset: {}, pos: {}", path, offset, pos);
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
        Ok(FnParser {
            start: offset,
            buf: buf
        })
    }

    pub fn scope(&self) -> Scope {

        let ifn = self.buf.rfind('.').map(|n| n+1).unwrap_or(0);
        let ipath = self.buf.rfind(':').map(|n| n+1).unwrap_or(0);
        let iword = self.buf.rfind(|c: char| !(c.is_alphabetic() || c.is_numeric() || c == '_'))
            .map(|n| n+1).unwrap_or(0);

        if ifn > ipath {
            if ifn > iword {
                Scope::Fn(self.buf[iword..].split('.')
                    .map(|s| Token { name: s.to_string(), pos: self.start + ifn })
                    .collect())
            } else {
                Scope::Word(Token { name: self.buf[iword..].to_string(), pos: self.start + iword })
            }
        } else {
            if ipath > iword {
                Scope::Path(self.buf[iword..].split("::")
                    .map(|s| Token { name: s.to_string(), pos: self.start + ipath })
                    .collect())
            } else {
                Scope::Word(Token { name: self.buf[iword..].to_string(), pos: self.start + iword })
            }
        }
    }

    pub fn iter<'a>(&'a self, name: &'a str, end: usize) -> FnIter<'a> {
        let buf_end = if end < self.start { 0 } else { end - self.start };
        FnIter {
           inner: &self,
           name: name,
           buf_end: buf_end
        }
    }
}

pub struct FnIter<'a> {
    inner: &'a FnParser,
    name: &'a str,
    buf_end: usize
}

impl<'a> Iterator for FnIter<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        loop {
            // TODO: extend until end of found word
            match self.inner.buf[..self.buf_end].rfind(self.name).map(|n| {
                self.buf_end = n;
                Token { name: self.name.to_string(), pos: self.inner.start + self.buf_end }
            }) {
                Some(t) => {
                    let start = self.inner.buf[..self.buf_end].rfind('\n').map(|n|n+1).unwrap_or(0);
                    let end = self.buf_end + self.inner.buf[self.buf_end..].find('\n').unwrap_or(0);
                    let line = &self.inner.buf[start..end];
                    debug!("search for token: {:?} in line:\n{}", t, line);
                    if REGEX_DEF.is_match(line) || REGEX_ARG.is_match(line) { return Some(t) }
                    self.buf_end = start;
                },
                None => {
                    debug!("can't find matching name: {}", self.name);
                    return None;
                }
            }
        }
    }

}
