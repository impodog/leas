use super::*;

#[derive(Debug)]
pub struct Env {
    pub modules: Vec<PathBuf>,
    pub linking: Vec<PathBuf>,
    base_path: RefCell<Vec<PathBuf>>,
    imported: RefCell<HashMap<PathBuf, WeakResource>>,
}

impl Env {
    fn split_var(var: String) -> Vec<PathBuf> {
        var.split(':')
            .map(|s| Path::new(s).to_path_buf())
            .collect::<Vec<_>>()
    }

    pub fn read() -> Self {
        let mut modules: Vec<PathBuf> = Vec::new();
        let mut linking: Vec<PathBuf> = Vec::new();
        modules.push(Path::new(".").to_path_buf());
        linking.push(Path::new(".").to_path_buf());

        modules.extend(
            std::env::var("LEAS_PATH")
                .map(Self::split_var)
                .unwrap_or_default(),
        );
        linking.extend(
            std::env::var("LD_LIBRARY_PATH")
                .map(Self::split_var)
                .unwrap_or_default(),
        );

        Self {
            modules,
            linking,
            base_path: RefCell::new(vec![Path::new(".").to_path_buf()]),
            imported: RefCell::new(HashMap::new()),
        }
    }

    pub fn find_module(&self, name: &str) -> Option<PathBuf> {
        let name_with_suffix = Self::add_suffix(Path::new(name).to_path_buf());
        for (index, module) in self.modules.iter().enumerate() {
            let module = if index == 0 {
                self.base_path()
            } else {
                module.to_path_buf()
            };

            let path = module.join(&name);
            if path.exists() {
                return Some(Self::locate_module(path));
            }
            let path = module.join(&name_with_suffix);
            if path.exists() {
                return Some(Self::locate_module(path));
            }
        }
        None
    }

    pub fn find_library(&self, name: &str) -> Option<PathBuf> {
        for library in &self.linking {
            let path = library.join(name);
            if path.exists() {
                return path.canonicalize().ok();
            }
        }
        None
    }

    pub fn forward_base(&self, path: PathBuf) {
        if path.is_dir() {
            self.base_path.borrow_mut().push(path);
        } else {
            self.base_path
                .borrow_mut()
                .push(path.parent().unwrap().to_path_buf());
        }
    }

    pub fn backward_base(&self) {
        if self.base_path.borrow().len() > 1 {
            self.base_path.borrow_mut().pop();
        }
    }

    pub fn base_path(&self) -> PathBuf {
        self.base_path.borrow().last().unwrap().to_path_buf()
    }

    pub fn get_import(&self, name: &Path) -> Option<Resource> {
        self.imported.borrow().get(name)?.upgrade()
    }

    pub fn set_import(&self, name: PathBuf, res: &Resource) {
        self.imported.borrow_mut().insert(name, res.downgrade());
    }

    fn add_suffix(path: PathBuf) -> PathBuf {
        if path.extension().map_or(false, |ext| ext == "lea") {
            path
        } else {
            path.with_extension("lea")
        }
    }

    fn locate_module(path: PathBuf) -> PathBuf {
        if path.is_dir() {
            path.join("mod.lea").canonicalize().unwrap_or(path)
        } else {
            path.canonicalize().unwrap_or(path)
        }
    }
}
