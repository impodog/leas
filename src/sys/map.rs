use super::*;

#[derive(Debug)]
pub struct Map {
    data: HashMap<String, Value>,
    pushed: HashMap<String, Vec<Value>>,
    line: Rc<Cell<usize>>,

    env: Rc<Env>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            pushed: HashMap::new(),
            line: Rc::new(Cell::new(1)),

            env: Rc::new(Env::read()),
        }
    }

    pub fn new_under(map: &Map) -> Self {
        Self {
            data: HashMap::new(),
            pushed: HashMap::new(),
            line: map.line.clone(),

            env: map.env.clone(),
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

    pub fn push(&mut self, k: &str, v: Value) {
        let value = self.rem(k);
        if let Some(value) = value {
            match self.pushed.get_mut(k) {
                Some(pushed) => {
                    pushed.push(value);
                }
                None => {
                    self.pushed.insert(k.to_string(), vec![value]);
                }
            }
        }
        self.set(k.to_string(), v);
    }

    pub fn pop(&mut self, k: &str) {
        match self.pushed.get_mut(k) {
            Some(pushed) => {
                if let Some(value) = pushed.pop() {
                    self.set(k.to_string(), value);
                }
            }
            None => {
                self.rem(k);
            }
        }
    }

    pub fn req(&self, k: &str) -> Result<Value> {
        self.get(k)
            .ok_or_else(|| Error::new(format!("Required value {} is not found", k), self.line()))
            .cloned()
    }

    pub fn register(&mut self, name: &str, f: Func) {
        self.set(name.to_string(), Value::Res(Resource::new_func(f)));
    }

    pub fn set_line(&mut self, line: usize) {
        self.line.set(line);
    }

    pub fn line(&self) -> usize {
        self.line.get()
    }

    pub fn env(&self) -> &Env {
        self.env.as_ref()
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}
