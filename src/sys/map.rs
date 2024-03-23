use super::*;

#[derive(Debug)]
pub struct Map {
    data: HashMap<String, Value>,
    pushed: HashMap<String, Vec<Value>>,
    snapshot: Vec<(HashSet<String>, HashSet<String>)>,
    line: Rc<Cell<usize>>,

    env: Rc<Env>,

    parent: Option<Box<Map>>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            pushed: HashMap::new(),
            snapshot: Vec::new(),
            line: Rc::new(Cell::new(1)),

            env: Rc::new(Env::read()),

            parent: None,
        }
    }

    pub fn new_under(map: &Map) -> Self {
        Self {
            data: HashMap::new(),
            pushed: HashMap::new(),
            snapshot: Vec::new(),
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

    pub fn forced_set(&mut self, k: String, v: Value) -> Option<Value> {
        self.data.insert(k, v)
    }

    pub fn set(&mut self, k: String, v: Value) {
        if let Some((changes, globals)) = self.snapshot.last_mut() {
            if globals.contains(&k) {
                self.forced_set(k, v);
            } else {
                if changes.insert(k.clone()) {
                    self.push(k, v);
                } else {
                    self.forced_set(k, v);
                }
            }
        } else {
            self.forced_set(k, v);
        };
    }

    pub fn forced_rem(&mut self, k: &str) -> Option<Value> {
        self.data.remove(k)
    }

    pub fn rem(&mut self, k: Cow<str>) -> Option<Value> {
        if let Some((changes, globals)) = self.snapshot.last_mut() {
            if globals.contains(k.as_ref()) {
                self.forced_rem(k.as_ref())
            } else {
                if changes.insert(k.to_string()) {
                    if let Some(value) = self.get(k.as_ref()).cloned() {
                        self.push(k.into(), value.clone());
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    self.forced_rem(k.as_ref())
                }
            }
        } else {
            self.forced_rem(k.as_ref())
        }
    }

    pub fn snapshot(&mut self) {
        self.snapshot.push((HashSet::new(), HashSet::new()));
    }

    pub fn rollback(&mut self) {
        if let Some((snapshot, _)) = self.snapshot.pop() {
            for k in snapshot.into_iter() {
                self.pop(Cow::Owned(k));
            }
        }
    }

    pub fn global(&mut self, k: String) {
        if let Some((_, globals)) = self.snapshot.last_mut() {
            globals.insert(k);
        }
    }

    pub fn push(&mut self, k: String, v: Value) {
        let value = self.forced_rem(&k);
        if let Some(value) = value {
            match self.pushed.get_mut(&k) {
                Some(pushed) => {
                    pushed.push(value);
                }
                None => {
                    self.pushed.insert(k.to_string(), vec![value]);
                }
            }
        }
        self.forced_set(k, v);
    }

    pub fn push_name(&mut self, k: impl ToString, v: Value) {
        self.push(k.to_string(), v);
    }

    pub fn pop(&mut self, k: Cow<str>) {
        match self.pushed.get_mut(k.as_ref()) {
            Some(pushed) => {
                if let Some(value) = pushed.pop() {
                    self.set(k.into(), value);
                }
            }
            None => {
                self.rem(k);
            }
        }
    }

    pub fn pop_name(&mut self, k: &str) {
        self.pop(Cow::Borrowed(k));
    }

    pub fn req(&self, k: &str) -> Result<Value> {
        self.get(k)
            .ok_or_else(|| Error::new(format!("Required value {} is not found", k), self.line()))
            .cloned()
    }

    pub fn register(
        &mut self,
        name: impl ToString,
        f: impl FnMut(&mut Map, Value) -> Result<Value> + 'static,
    ) {
        self.forced_set(
            name.to_string(),
            Value::Res(Resource::new_func(Func::new(f, name.to_string()))),
        );
    }

    pub fn register_init(
        &mut self,
        name: impl ToString + fmt::Display,
        mut f: impl FnMut(&mut Map) + 'static,
    ) {
        self.forced_set(
            format!("_init_{}", name),
            Value::Res(Resource::new_func(Func::new(
                move |map, _| {
                    f(map);
                    Ok(Value::Null)
                },
                name.to_string(),
            ))),
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

    /// Link to a mutable reference of a map, leaving the original map in an invalid state.
    /// Remember to call `unlink_to` to get the original map back.
    pub unsafe fn link_to(&mut self, map: &mut Map) {
        self.link(std::mem::replace(
            map,
            #[allow(invalid_value)]
            std::mem::MaybeUninit::uninit().assume_init(),
        ))
    }

    pub fn unlink(&mut self) -> Option<Map> {
        self.parent.take().map(|p| *p)
    }

    /// Unlink the current map with its parent, moving the parent map to the given mutable reference.
    /// This function forgets the given map, and is *ONLY* used with `link_to`
    pub fn unlink_to(&mut self, map: &mut Map) {
        let unused_map = std::mem::replace(map, self.unlink().unwrap());
        std::mem::forget(unused_map);
    }

    pub fn parent(&self) -> Option<&Map> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    pub fn parent_mut(&mut self) -> Option<&mut Map> {
        self.parent.as_mut().map(|p| p.as_mut())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.data.iter()
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}
