use std::path::{Path, PathBuf};
use std::fs::PathExt;

mod cargo;

#[cfg(unix)]
pub const PATH_SEP: char = ':';
#[cfg(windows)]
pub const PATH_SEP: char = ';';

struct Module {
    name: String,
    path: PathBuf
}

impl Module {

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

}

pub struct Crate {
    root: Module,
    crates: Vec<Crate>,
    modules: Vec<Module>
}

impl Crate {

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
