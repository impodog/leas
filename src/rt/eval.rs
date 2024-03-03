use super::*;

pub trait Eval {
    fn eval(&self, map: &mut Map) -> Result<Value>;

    fn get(&self, map: &mut Map) -> Result<Value> {
        self.eval(map)
    }

    fn set(&self, map: &mut Map, value: Value) -> Result<Value> {
        self.eval(map)?
            .as_res()
            .ok_or_else(|| Error::new(format!("Cannot set value to {:?}", value), map.line()))?
            .visit(|s: &String| map.set(s.clone(), value.clone()));
        Ok(value)
    }
}
