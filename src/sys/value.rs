use super::*;

pub type Int = i64;
pub type Float = f64;
pub type Uint = u64;
pub type Bool = bool;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(Int),
    Float(Float),
    Uint(Uint),
    Bool(Bool),
    Null,
    Stop,
    Res(Resource),
    Weak(WeakResource),
}

impl Value {
    pub fn as_int(&self) -> Option<Int> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<Float> {
        match self {
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_uint(&self) -> Option<Uint> {
        match self {
            Self::Uint(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<Bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_null(&self) -> Option<()> {
        match self {
            Self::Null => Some(()),
            _ => None,
        }
    }

    pub fn as_res(&self) -> Option<Resource> {
        match self {
            Self::Res(res) => Some(res.clone()),
            Self::Weak(weak) => weak.upgrade(),
            _ => None,
        }
    }

    pub fn downgrade(self) -> Self {
        match self {
            Self::Res(res) => Self::Weak(res.downgrade()),
            _ => self,
        }
    }

    pub fn upgrade(self) -> Option<Self> {
        match self {
            Self::Weak(weak) => weak.upgrade().map(Self::Res),
            _ => Some(self),
        }
    }

    pub fn call(&self, map: &mut Map, value: Value) -> Option<Result<Value>> {
        map.push("self", self.clone());
        let result = self.as_res()?.visit_mut_func(|f| Some((f.f)(map, value)));
        map.pop("self");
        result?
    }
}

impl Value {
    pub fn visit_res_or_else<F, T, R, E>(&self, f: F, err: E) -> Result<R>
    where
        F: FnOnce(&T) -> R + 'static,
        T: Res + 'static,
        R: 'static,
        E: FnOnce() -> Error + Clone + 'static,
    {
        self.as_res()
            .ok_or_else(err.clone())?
            .visit(f)
            .ok_or_else(err)
    }

    pub fn visit_mut_res_or_else<F, T, R, E>(&self, f: F, err: E) -> Result<R>
    where
        F: FnOnce(&mut T) -> R + 'static,
        T: Res + 'static,
        R: 'static,
        E: FnOnce() -> Error + Clone + 'static,
    {
        self.as_res()
            .ok_or_else(err.clone())?
            .visit_mut(f)
            .ok_or_else(err)
    }
}
