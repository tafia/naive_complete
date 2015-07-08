# naive_complete

This project (name may change) aims at providing a fast, **simple**
autocompletion tool for [rust](http://www.rust-lang.org/).

## Principles

This is a very naive (simple and understandable) attempt and does not plan to
compete with better tools such as [racer](https://github.com/phildawes/racer).
Main guidelines:
- **iterator**: to allow very lazy `find-definition`
- **regexes**: while perhaps not best in class in terms of performance,
drastically reduces code complexity
- **not exact**: this project will probably *never* rely on the rustc compiler.
  In particular there may be some special corner cases never implemented.
  It's totally fine as long as *most* work
  (ie it is good enough to be used as a autocomplete tool)

Eventually it'll at least provide benchmarks to compare with.

## Architecture (planned)

As of now, I foresee 4 modules
- [main](https://github.com/tafia/naive_complete/blob/master/src/main.rs)
  - work as an Arena and store/digest outputs from the 3 other modules
  - as lazy as possible, responsible for type-check/search
  - should probably allow any file to be parsed once only
  - largely not implemented
- [*File* Parser iterator](https://github.com/tafia/naive_complete/blob/master/src/file_parser):
  - take a file path as input and provides an iterator over items relevant for
  the autocompletion
  - in particular, do/may not parse
    - comments, attributes (`#[...]`)
    - `fn` bodies,
    - `impl` bodies (depends if relevant or not)
    - non-`pub` items if looking for an external file
  - there is already a good-enough-to-start version with the `SearchIter`
  - can probably be extended to work with any stream as it is based on a
  `BufReader` ... so there is no need to *write* a temporary file
- [*Function* Parser iterator](https://github.com/tafia/naive_complete/blob/master/src/func_parser.rs):
  - there will be at maximum ONE function body parsed (the one currently in scope)
  - provide an iterator over fn variables
  - basic implementation, which use a private file reader for the moment
- [File Searcher iterator](https://github.com/tafia/naive_complete/blob/master/src/file_searcher.rs):
  - iterate over possible files in crates
  - defined by `use` statements, `extern crate` (including prelude) ...
  - can discover from cargo files
  - basic implementation
  - relies on [racer cargo file](https://github.com/tafia/naive_complete/blob/master/src/file_parser/cargo.rs)( to search in dependencies

## Todo

- Implement basic workable version
- Add tests + eventually benchmarks
- Add additional properties to skip some parsing in file parser (local,
  static fns in particular)
- Perhaps use Cow in Token
- Check BufReader (see this [SO](https://www.reddit.com/r/rust/comments/3cgaui/trying_to_find_why_python_is_twice_as_fast_as/))
