use std::path::{Path, PathBuf};
use std::fs::PathExt;

enum Use {
    Fn(Vec<String>),
    Object(Vec<String>),
    Blob(Vec<String>)
}

struct Module {
    name: String,
    path: PathBuf
}

impl Module {
    fn new(parent: &Path, name: &str) -> Option<Module> {

        let path = if parent.is_file() {
            parent.parent().unwrap()
        } else { parent };

        // try <name.rs>
        for p in [format!("{}.rs", name),
                  format!("{}/mod.rs", name),
                  format!("{}/{}.rs", name, name),
                  format!("{}/lib.rs", name)].into_iter() {
            let mod_path = path.join(p);
            if mod_path.exists() {
                return Some(Module {
                    name: name.to_string(),
                    path: mod_path
                })
            }
        }

        None
    }
}

pub struct Crate {
    root: Module,
    crates: Vec<Crate>,
    modules: Vec<Module>
}

impl Crate {
    pub fn new(parent: &Path, name: &str) -> Option<Crate> {
        // See racer/matchers#188
        Module::new(parent, name).map(|m| Crate {
            root: m,
            crates: Vec::new(),
            modules: Vec::new()
        })
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

    pub fn find_use_file(&self, file: &str, use_stmt: &str) -> String {
        String::new()
    }
}
