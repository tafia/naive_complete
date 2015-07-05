enum Use {
	Fn(Vec<String>),
	Object(Vec<String>),
	Blob(Vec<String>)
}

struct Module {
	name: String,
	path: String
}

impl Module {
	fn new(parent: &str, name: &str) -> Module {
		// See racer/matchers#211
		Module {
			name: name.to_string(),
			path: parent.to_string() + name		// totally wrong
		}
	}
}

pub struct Crate {
	root: Module,
	crates: Vec<Crate>,
	modules: Vec<Module> 
}

impl Crate {
	pub fn new(parent: &str, name: &str) -> Crate {
		// See racer/matchers#159
		Crate {
			root: Module::new(parent, name),
			crates: Vec::new(),
			modules: Vec::new()
		}
	}

	pub fn add_crate(&mut self, krate: &str) {
		self.crates.push(Crate::new(&self.root.path, krate));
	}

	pub fn add_module(&mut self, module: &str) {
		self.modules.push(Module::new(&self.root.path, module))
	}

	pub fn find_use_file(&self, file: &str, use_stmt: &str) -> String {
		String::new()
	}
}