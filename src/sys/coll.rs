use super::*;

pub struct Func {
    pub f: Box<dyn Fn(&mut Map, VecDeque<Value>) -> Result<Value>>,
}

impl Func {
    pub fn new(f: impl Fn(&mut Map, VecDeque<Value>) -> Result<Value> + 'static) -> Self {
        Self { f: Box::new(f) }
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Dynamic Function>")
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
