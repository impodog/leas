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
    Res(Resource),
}
