use std::io::{BufRead, BufReader, Result};
use std::fs::File;
use regex::Regex;
use std::str::from_utf8;

static REGEX_LET: Regex = regex!(r"(?:if\s+)?let\s+(\w+)\s+=");

#[derive(Debug,Clone,PartialEq)]
struct Token {
    name: String,      // match name
    pos: usize         // position in the file
}

struct SearchFnIter {
    file: BufReader<File>,
    pos: usize,
    skip: Option<(u8, u8)>,
    buf: String
}
