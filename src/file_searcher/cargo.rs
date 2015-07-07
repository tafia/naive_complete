// This file  has been taken from racer source code
// https://github.com/phildawes/racer

use std::io::Read;
use std::env;
use std::path::{Path,PathBuf};
use std::fs::{File, PathExt, read_dir};
use toml;

// otry is 'option try'
macro_rules! otry {
    ($e:expr) => (match $e { Some(e) => e, None => return None })
}

fn read_to_string(file: &Path) -> Option<String> {
    File::open(file).ok().and_then(|mut file| {
        let mut string = String::new();
        file.read_to_string(&mut string).ok().map(|_| string)
    })
}

fn find_src_via_lockfile(kratename: &str, cargofile: &Path) -> Option<PathBuf> {
    let string = otry!(read_to_string(cargofile));
    let mut parser = toml::Parser::new(&string);
    let lock_table = parser.parse().unwrap();

    debug!("find_src_via_lockfile found lock table {:?}", lock_table);

    let t = match lock_table.get("package") {
        Some(&toml::Value::Array(ref t1)) => t1,
        _ => return None
    };

    for item in t {
        if let &toml::Value::Table(ref t) = item {
            if Some(&toml::Value::String(kratename.to_string())) == t.get("name") {
                let version = otry!(getstr(t, "version"));
                let source = otry!(getstr(t, "source"));

                if Some("registry") == source.split("+").nth(0) {
                    return get_versioned_cratefile(kratename, &version);
                } else if Some("git") == source.split("+").nth(0) {
                    let sha1 = otry!(source.split("#").last());
                    let mut d = otry!(env::home_dir());
                    d.push(".cargo");
                    d.push("git");
                    d.push("checkouts");
                    d = otry!(find_git_src_dir(d, kratename, &sha1));
                    d.push("src");
                    d.push("lib.rs");
                    return Some(d);
                }
            }
        }
    }

    None
}

fn get_versioned_cratefile(kratename: &str, version: &str) -> Option<PathBuf> {
    env::home_dir().and_then(|mut d| {
        d.push(".cargo");
        d.push("registry");
        d.push("src");
        d = otry!(find_cratesio_src_dir(d));
        d.push(kratename.to_string() + "-" + &version);
        d.push("src");
        d.push("lib.rs");
        Some(d)
    })
 }

fn find_src_via_tomlfile(kratename: &str, cargofile: &Path) -> Option<PathBuf> {
    // only look for 'path' references here.
    // We find the git and crates.io stuff via the lockfile

    let string = otry!(read_to_string(cargofile));
    let mut parser = toml::Parser::new(&string);
    let table = otry!(parser.parse());
    let t = match table.get("dependencies") {
        Some(&toml::Value::Table(ref t)) => t,
        _ => return None
    };

    match t.get(kratename) {
        Some(&toml::Value::Table(ref t)) => {
            // local directory
            let relative_path = otry!(getstr(t, "path"));
            Some(otry!(cargofile.parent())
                        .join(relative_path)
                        .join("src")
                        .join("lib.rs"))
        },
        Some(&toml::Value::String(ref version)) =>
            // versioned crate
            get_versioned_cratefile(kratename, version),
        _ => None
    }
}

fn find_cratesio_src_dir(d: PathBuf) -> Option<PathBuf> {
    for entry in otry!(read_dir(d).ok()) {
        let path = otry!(entry.ok()).path();
        if path.exists() {
            if let Some(ref fname) = path.file_name().and_then(|s| s.to_str()) {
                if fname.starts_with("github.com-") {
                    return Some(path.clone());
                }
            }
        }
    }
    None
}

fn find_git_src_dir(d: PathBuf, name: &str, sha1: &str) -> Option<PathBuf> {
    for entry in otry!(read_dir(d).ok()) {
        let path = otry!(entry.ok()).path();
        if path.is_dir() {
            if let Some(ref fname) = path.file_name().and_then(|s| s.to_str()) {
                if fname.starts_with(name) {
                    let mut d = path.clone();

                    // dirname can be the sha1 or master.
                    d.push(sha1);

                    if d.is_dir() {
                        d.pop();
                        d.push("master");
                    }

                    let retval = d.clone();

                    // check that the checkout matches the commit sha1
                    d.push(".git");
                    d.push("refs");
                    d.push("heads");
                    d.push("master");

                    let mut headref = String::new();
                    otry!(otry!(File::open(d).ok()).read_to_string(&mut headref).ok());

                    debug!("git headref is {:?}", headref);

                    if headref.ends_with("\n") {
                        headref.pop();
                    }

                    if sha1 == headref {
                        return Some(retval);
                    }
                }
            }
        }
    }
    None
}

fn getstr(t: &toml::Table, k: &str) -> Option<String> {
    match t.get(k) {
        Some(&toml::Value::String(ref s)) => Some(s.clone()),
        _ => None
    }
}

fn find_cargo_tomlfile(currentfile: &Path) -> Option<PathBuf> {
    let mut f = currentfile.to_path_buf();
    f.push("Cargo.toml");
    if f.exists() {
        Some(f)
    } else {
        if f.pop() && f.pop() {
            find_cargo_tomlfile(&f)
        } else {
            None
        }
    }
}

pub fn get_crate_file(kratename: &str, from_path: &Path) -> Option<PathBuf> {
    find_cargo_tomlfile(from_path).and_then(|tomlfile| {
        // look in the lockfile first, if there is one
        debug!("get_crate_file tomlfile is {:?}", tomlfile);
        let mut lockfile = tomlfile.clone();
        lockfile.pop();
        lockfile.push("Cargo.lock");
        if lockfile.exists() {
            if let Some(f) = find_src_via_lockfile(kratename, &lockfile) {
                return Some(f);
            }
        }

        // oh, no luck with the lockfile. Try the tomlfile
        find_src_via_tomlfile(kratename, &tomlfile)
    })
}
