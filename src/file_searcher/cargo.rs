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

// converts errors into None
macro_rules! otry2 {
    ($e:expr) => (match $e { Ok(e) => e, Err(e) => {
        error!("ERROR!: {:?} {} {}", e, file!(), line!());
        return None
    } })
}

fn read_to_string(file: &Path) -> Option<String> {
    File::open(file).ok().and_then(|mut file| {
        let mut string = String::new();
        file.read_to_string(&mut string).ok().map(|_| string)
    })
}

fn find_src_via_lockfile(kratename: &str, cargofile: &Path) -> Option<PathBuf> {

    if !cargofile.exists() { return None; }

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
            if let Some(&toml::Value::String(ref name)) = t.get("name") {
                if name.replace("-", "_") == kratename {
                    debug!("found matching crate {:?}", t);
                    let version = otry!(getstr(t, "version"));
                    let source = otry!(getstr(t, "source"));

                    if Some("registry") == source.split("+").nth(0) {
                        return get_versioned_cratefile(name, &version);
                    } else if Some("git") == source.split("+").nth(0) {
                        let sha1 = otry!(source.split("#").last());
                        let mut d = otry!(get_cargo_rootdir());
                        d.push("git");
                        d.push("checkouts");
                        d = otry!(find_git_src_dir(d, name, &sha1));
                        d.push("src");
                        d.push("lib.rs");
                        return Some(d);
                    }
                }
            }
        }
    }
    None
}

fn get_cargo_rootdir() -> Option<PathBuf> {

    if let Some(x) = env::var_os("CARGO_HOME") {
        let d = PathBuf::from(x);
        return if d.exists() { Some(d) } else { None };
    }

    let mut d = otry!(env::home_dir());

    // try multirust first, since people with multirust installed will often still
    // have an old .cargo directory lying around
    d.push(".multirust");
    d.push("default");
    if let Ok(mut multirustdefault) = File::open(&d) {
        let mut s = String::new();
        otry2!(multirustdefault.read_to_string(&mut s));
        d.pop();
        d.push("toolchains");
        d.push(s.trim());
        d.push("cargo");
        debug!("get_cargo_rootdir root is {:?}",d);
        return Some(d)
    }

    d.pop();
    d.pop();
    d.push(".cargo");
    if d.exists() { Some(d) } else { None }
}

fn get_versioned_cratefile(kratename: &str, version: &str) -> Option<PathBuf> {
    let mut d = otry!(get_cargo_rootdir());
    d.push("registry");
    d.push("src");
    d = otry!(find_cratesio_src_dir(d));

    // if version=* then search for the first matching folder
    if version == "*" {
        let mut start_path = d.clone();
        start_path.push(kratename);
        let start_name = start_path.to_str().unwrap();

        if let Ok(reader) = read_dir(d) {
            if let Some(path) = reader
                                .map(|entry| entry.unwrap().path())
                                .find(|path| path.to_str().unwrap().starts_with(start_name)) {
                d = path.clone();
            } else {
                return None;
            }
        } else { return None; }
    } else {
        d.push(kratename.to_string() + "-" + &version);
    }

    d.push("src");
    debug!("crate path {:?}",d);

    // First, check for package name at root (src/kratename/lib.rs)
    d.push(kratename.to_string());
    d.push("lib.rs");
    if let Err(_) = File::open(&d) {
        // It doesn't exist, so assume src/lib.rs
        d.pop();
        d.pop();
        d.push("lib.rs");
    }
    debug!("crate path with lib.rs {:?}",d);

    if File::open(&d).is_ok() { Some(d) } else { None }
 }

fn find_src_via_tomlfile(kratename: &str, cargofile: &Path) -> Option<PathBuf> {
    // only look for 'path' references here.
    // We find the git and crates.io stuff via the lockfile

    let string = otry!(read_to_string(cargofile));
    let mut parser = toml::Parser::new(&string);
    let table = otry!(parser.parse());

    // is it this lib?  (e.g. you're searching from tests to find the main library crate)
    if let Some(&toml::Value::Table(ref t)) = table.get("lib") {
        if let Some(&toml::Value::String(ref name)) = t.get("name") {
            if name == kratename {
                debug!("found {} as lib entry in Cargo.toml", kratename);
                if let Some(&toml::Value::String(ref pathstr)) = t.get("path") {
                    let p = Path::new(pathstr);
                    let libpath = otry!(cargofile.parent()).join(p);
                    return Some(libpath);
                }
            }
        }
    }

    // otherwise search the dependencies
    let t = match table.get("dependencies") {
        Some(&toml::Value::Table(ref t)) => t,
        _ => return None
    };

    let mut name = kratename;
    let value = if kratename.contains('_') {
        t.iter().find(|&(k, _)| k.replace("-", "_") == name).map(|(k,v)| {
            name = k;
            v
        })
    } else {
        t.get(kratename)
    };

    match value {
        Some(&toml::Value::Table(ref t)) => {
            // local directory
            let relative_path = otry!(getstr(t, "path"));
            Some(otry!(cargofile.parent())
                .join(relative_path)
                .join("src")
                .join("lib.rs"))
        },
        Some(&toml::Value::String(ref version)) => {
            // versioned crate
            get_versioned_cratefile(name, version)
        }
        _ => None
    }
}

fn find_cratesio_src_dir(d: PathBuf) -> Option<PathBuf> {
    for entry in otry2!(read_dir(d)) {
        let path = otry2!(entry).path();
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
    for entry in otry2!(read_dir(d)) {
        let path = otry2!(entry).path();
        if path.is_dir() {
            if let Some(ref fname) = path.file_name().and_then(|s| s.to_str()) {
                if fname.starts_with(name) {
                    let mut d = path.clone();

                    // dirname can be the sha1 or master.
                    d.push(sha1);

                    if !d.is_dir() {
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
                    otry2!(otry2!(File::open(d)).read_to_string(&mut headref));

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

pub fn find_cargo_tomlfile(currentfile: &Path) -> Option<PathBuf> {
    let mut f = currentfile.to_path_buf();
    loop {
        f.push("Cargo.toml");
        if f.exists() { return Some(f); }
        if !f.pop() || !f.pop() { return None; }
    }
}

pub fn get_crate_file(kratename: &str, from_path: &Path) -> Option<PathBuf> {
    find_cargo_tomlfile(from_path).and_then(|tomlfile| {
        // look in the lockfile first, if there is one
        debug!("get_crate_file tomlfile is {:?}", tomlfile);
        let mut lockfile = tomlfile.clone();
        lockfile.pop();
        lockfile.push("Cargo.lock");

        find_src_via_lockfile(kratename, &lockfile)
        // oh, no luck with the lockfile. Try the tomlfile
        .or(find_src_via_tomlfile(kratename, &tomlfile))
    })
}
