use super::*;

#[derive(Debug)]
pub struct Map {
    data: HashMap<String, Value>,
    line: usize,
}

impl Map {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            line: 1,
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

    pub fn set_line(&mut self, line: usize) {
        self.line = line;
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}
