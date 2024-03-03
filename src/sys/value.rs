use super::*;

pub type Int = i64;
pub type Float = f64;
pub type Uint = u64;
pub type Bool = bool;

#[derive(Debug, Clone)]
pub enum Value {
    Int(Int),
    Float(Float),
    Uint(Uint),
    Bool(Bool),
    Null,
    Stop,
    Res(Resource),
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

    pub fn as_res(&self) -> Option<&Resource> {
        match self {
            Self::Res(res) => Some(res),
            _ => None,
        }
    }

    pub fn call(&self, map: &mut Map, value: VecDeque<Value>) -> Option<Result<Value>> {
        self.as_res()?
            .visit_mut_func(move |f| Some((f.f)(map, value)))?
    }
}
