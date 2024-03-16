use super::*;

#[derive(Debug)]
pub struct Map {
    data: HashMap<String, Value>,
    pushed: HashMap<String, Vec<Value>>,
    line: Rc<Cell<usize>>,

    env: Rc<Env>,

    parent: Option<Box<Map>>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            pushed: HashMap::new(),
            line: Rc::new(Cell::new(1)),

            env: Rc::new(Env::read()),

            parent: None,
        }
    }

    pub fn new_under(map: &Map) -> Self {
        Self {
            data: HashMap::new(),
            pushed: HashMap::new(),
            line: map.line.clone(),

            env: map.env.clone(),

            parent: None,
        }
    }

    pub fn get(&self, k: &str) -> Option<&Value> {
        self.data
            .get(k)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(k)))
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

    pub fn register(
        &mut self,
        name: &str,
        f: impl FnMut(&mut Map, Value) -> Result<Value> + 'static,
    ) {
        self.set(
            name.to_string(),
            Value::Res(Resource::new_func(Func::new(f))),
        );
    }

    pub fn register_init(&mut self, name: &str, mut f: impl FnMut(&mut Map) + 'static) {
        self.set(
            format!("_init_{}", name),
            Value::Res(Resource::new_func(Func::new(move |map, _| {
                f(map);
                Ok(Value::Null)
            }))),
        );
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

    pub fn link(&mut self, parent: Map) {
        self.parent = Some(Box::new(parent));
    }

    pub fn unlink(&mut self) -> Option<Map> {
        self.parent.take().map(|p| *p)
    }

    pub fn unlink_to(&mut self, map: &mut Map) {
        let _ = std::mem::replace(map, self.unlink().unwrap());
    }

    pub fn parent(&self) -> Option<&Map> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    pub fn parent_mut(&mut self) -> Option<&mut Map> {
        self.parent.as_mut().map(|p| p.as_mut())
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}
