use func_parser::{FnParser, Scope};
use file_searcher::{Module, ModuleIter};
use file_parser::Searcheable;

#[derive(Debug,Clone,PartialEq)]
pub struct Token {
    pub name: String,   // match name
    pub pos: usize      // position in the file
}

// find-definition pos fname
pub fn find_definition(file: &str, pos: usize) -> Option<Token> {

    Module::root(file).and_then(|module| {

        let mut mod_iter = module.iter();

        // search for fn start (offset)
        let mut offset = 0;
        let _ = mod_iter.find(|s| {
            let end = s.get_pos();
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

            // smaller to bigger scope searches
            find_def_in_fn(&first_word, &inner_scope)
            .or(find_def_in_file(&first_word, &mut mod_iter))
            // ... need to search for external files
            .or(find_def_in_use(&first_word, &mut mod_iter))
        })

    })

}

fn find_def_in_fn(word: &Token, fn_parser: &FnParser) -> Option<Token> {
    fn_parser.iter(&word.name, word.pos).find(|t| t.name.starts_with(&word.name))
}

fn find_def_in_file(word: &Token, mod_iter: &mut ModuleIter) -> Option<Token> {
    mod_iter.reset();
    mod_iter.into_iter()
    .filter_map(|s| {
        let t = s.get_main_token();
        if t.name.starts_with(&word.name) {
            Some((*t).clone())
        } else {
            None
        }
    }).next()
}

fn find_def_in_use(word: &Token, mod_iter: &mut ModuleIter) -> Option<Token> {
    mod_iter.reset();
    mod_iter.into_iter().filter_map(|s|
        match s {
            Searcheable::Use(path, name) => {
                // let file = path.push()
                None
            }
            _ => None
        }
    ).next()
}
