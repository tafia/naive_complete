use std::path::{Path, PathBuf};
use std::fs::{PathExt, read_dir};
use std::vec::IntoIter;
use std::slice::Iter;
use std::io::{Result, Error, ErrorKind};

use manager::Token;
use file_parser::{Searcheable, SearchIter};

mod cargo;

use self::cargo::find_cargo_tomlfile;

#[cfg(unix)]
pub const PATH_SEP: char = ':';
#[cfg(windows)]
pub const PATH_SEP: char = ';';

#[derive(Clone)]
pub struct Module {
    name: String,
    path: PathBuf
}

impl Module {

    pub fn root(file: &str) -> Option<Module> {
        let path = PathBuf::from(file);
        if path.exists() {
            Some(Module {
                name: path.file_name().unwrap().to_str().unwrap().to_string(),
                path: path
            })
        } else {
            None
        }
    }

    fn new(parent: &Path, name: &str) -> Option<Module> {

        let path =
            if parent.is_file() { parent.parent().unwrap() }
            else { parent };

        [format!("{}.rs", name),
         format!("{}/mod.rs", name),
         format!("{0}/{0}.rs", name),
         format!("{}/lib.rs", name)]
        .into_iter().map(|p| path.join(p))
        .find(|mod_path| mod_path.exists())
        .map(|mod_path| Module {
            name: name.to_string(),
            path: mod_path
        })

    }

    pub fn iter(&self) -> ModuleIter {
        ModuleIter {
            items: Vec::new(),
            iter: SearchIter::open(self.path.to_str().unwrap()).unwrap(),
            index: 0
        }
    }

}

pub struct ModuleIter {
    items: Vec<Searcheable>,
    iter: SearchIter,
    index: usize
}

impl ModuleIter {
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

impl Iterator for ModuleIter {
    type Item = Searcheable;

    fn next(&mut self) -> Option<Searcheable> {
        if self.index < self.items.len() {
            self.index += 1;
            return Some(self.items[self.index-1].clone());
        }

        let next = self.iter.next();
        if let Some(s) = next.clone() { self.items.push(s); }
        next
    }

}


pub struct Crate {
    root: Module,
    crates: Vec<Crate>,
    modules: Vec<Module>
}

impl Crate {

    pub fn root_module(module: &Module, iter: &ModuleIter) -> Result<Crate> {
        // need to find the file with the "main" fn as the crate root
        iter.reset();
        if iter.any(|s| match s {
            Searcheable::Fn(Token {name: name, ..}, _) => name == "main",
            _ => false
        }) {
            Ok(Crate {
                root: (*module).clone(),
                crates: Vec::new(),
                modules: Vec::new()
            })
        } else {
            if let Some(file) = find_cargo_tomlfile(&*module.path) {
                file.pop();
                file.push("src");
                if file.exists() {
                    for f in try!(read_dir(file)) {
						let path = try!(f).path();
						if path.extension().unwrap() == "rs" && 
							!path.starts_with(&*module.path) {
							let f_module = Module::root(path.to_str().unwrap());
							if f_module.iter.any(|s| match s {
								Searcheable::Fn(Token {name: name, ..}, _) => name == "main",
								_ => false
							}) return Ok(Crate {
								root: f_module,
								crates: Vec::new(),
								modules: Vec::new()
							})
						}
					}
					Err(ErrorKind::Other, "Cannot find rust file with main fn")
                }else {
                    Err(ErrorKind::Other, "Cannot find src directory")
                }
            } else {
				Err(ErrorKind::Other, "Cannot find cargo toml file")
			}
        }
    }

    pub fn new(parent: &Path, name: &str) -> Option<Crate> {
        cargo::get_crate_file(name, parent)
        .or(Crate::get_rust_crate(name))
        .and_then(|krate|
            Module::new(&krate, name).map(|m| Crate {
                root: m,
                crates: Vec::new(),
                modules: Vec::new()
            }))
    }

    pub fn add_crate(&mut self, name: &str) {
        if let Some(c) = Crate::new(&self.root.path, name) {
            self.crates.push(c);
        }
    }

    pub fn add_module(&mut self, name: &str) {
        if let Some(m) = Module::new(&self.root.path, name) {
            self.modules.push(m);
        }
    }

    fn get_rust_crate(name: &str) -> Option<PathBuf> {
        ::std::env::var("RUST_SRC_PATH").ok()
        .and_then(|rust_src| {
            let names = vec![format!("lib{}", name), name.to_string()];
            rust_src.split(PATH_SEP).into_iter()
            .flat_map(|s| names.iter().cloned().map(move |n|
                Path::new(s).join(n).join("lib.rs")).into_iter())
            .find(|filepath| filepath.exists())
        })
    }
}
