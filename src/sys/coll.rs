use super::*;

pub struct Func {
    pub f: Box<dyn FnMut(&mut Map, Value) -> Result<Value>>,
    pub name: Option<String>,
}

impl Func {
    pub fn new(f: impl FnMut(&mut Map, Value) -> Result<Value> + 'static, name: String) -> Self {
        Self {
            f: Box::new(f),
            name: Some(name),
        }
    }

    pub fn new_unnamed(f: impl FnMut(&mut Map, Value) -> Result<Value> + 'static) -> Self {
        Self {
            f: Box::new(f),
            name: None,
        }
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name {
            Some(ref name) => write!(f, "<fn {}>", name),
            None => write!(f, "<anonymous function>"),
        }
    }
}

impl Resource {
    pub fn new_func(f: Func) -> Self {
        Self::new(Box::new(f))
    }

    pub fn visit_func<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Box<Func>) -> R,
        R: 'static,
    {
        self.visit(f)
    }

    pub fn visit_mut_func<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Box<Func>) -> R,
        R: 'static,
    {
        self.visit_mut(f)
    }
}
