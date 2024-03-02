use super::*;

#[derive(Debug)]
pub struct Map {
    data: HashMap<String, Value>,
}

impl Map {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, k: &str) -> Option<&Value> {
        self.data.get(k)
    }

    pub fn set(&mut self, k: String, v: Value) -> Option<Value> {
        self.data.insert(k, v)
    }

    pub fn rem(&mut self, k: &str) -> Option<Value> {
        self.data.remove(k)
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource {
    pub fn new_map() -> Self {
        Self::new(Map::new())
    }

    pub fn visit_map<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Ref<Map>) -> R,
        R: 'static,
    {
        self.visit(f)
    }

    pub fn visit_mut_map<F, R>(&self, f: F) -> R
    where
        F: FnOnce(RefMut<Map>) -> R,
        R: 'static,
    {
        self.visit_mut(f)
    }
}
