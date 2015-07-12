use func_parser::{FnParser, Scope, FnIter};
use file_parser::{SearchIter, Searcheable};

#[derive(Debug,Clone,PartialEq)]
pub struct Token {
    pub name: String,   // match name
    pub pos: usize      // position in the file
}

// find-definition pos fname
pub fn find_definition(file: &str, pos: usize) -> Option<Token> {

    // search for all file entries up to requested `pos`, and save the offset
    let mut iter = SearchIter::open(file).unwrap();

    // get the scope search (Searcheable item)
    let mut offset = 0;
    let searcheable = iter.find(|s| {
        let end = match *s {
            Searcheable::Fn(Token {pos: p, ..}, _)      |
            Searcheable::Impl(_, Token {pos: p, ..}, _) |
            Searcheable::StructEnum(Token {pos: p, ..}) |
            Searcheable::Const(_, Token {pos: p, ..})   |
            Searcheable::Trait(Token {pos: p, ..})      => p,
            Searcheable::Use(_, ref v) => if v.len() > 0 { v[0].pos } else { 0 }
        };
        if end > pos { return true; }
        offset = end;
        false
    });

    // get the fn parser for the Searcheable item
    FnParser::new(file, offset, pos).ok().and_then(|inner_scope| {

        let scope = inner_scope.scope();
        debug!("root scope:\n{:?}", scope);

        let first_word = match scope {
            Scope::Path(ref segments) |
            Scope::Fn(ref segments)   => &segments[0],
            Scope::Word(ref word) => word
        }.clone();

        if first_word.name.len() == 0 {
            debug!("No word to be found");
            return None
        }

        find_def_in_fn(&first_word, &inner_scope)
        .or(find_def_in_file(&first_word, &mut SearchIter::open(file).unwrap()))
    })

}

fn find_def_in_fn(word: &Token, fn_parser: &FnParser) -> Option<Token> {
    fn_parser.iter(&word.name, word.pos).find(|t| t.name.starts_with(&word.name))
}

fn find_def_in_file(word: &Token, file_parser: &mut SearchIter) -> Option<Token> {
    file_parser.into_iter().map(move |s| match s {
        Searcheable::Fn(t, _)       |
        Searcheable::Impl(_, t, _)  |
        Searcheable::StructEnum(t)|
        Searcheable::Const(t, _)       |
        Searcheable::Trait(t)       => t,
        Searcheable::Use(_, _) => Token {name: "".to_string(), pos: 0}
    }).find(|t| t.name.starts_with(&word.name))
}
