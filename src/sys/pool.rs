use super::*;

#[derive(Debug, Clone)]
pub struct Resource(Rc<dyn Res>);

pub trait Res
where
    Self: fmt::Debug,
{
    fn as_any(&self) -> &dyn Any;
}

impl<T> Res for T
where
    Self: fmt::Debug + Sized + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Resource {
    pub fn new<T>(value: T) -> Self
    where
        T: Res + Sized + 'static,
    {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn visit<T, F, R>(&self, f: F) -> Option<R>
    where
        T: 'static,
        F: FnOnce(&T) -> R,
        R: 'static,
    {
        Some(f(&self
            .0
            .as_ref()
            .as_any()
            .downcast_ref::<RefCell<T>>()?
            .borrow()))
    }

    pub fn visit_mut<T, F, R>(&self, f: F) -> Option<R>
    where
        T: 'static,
        F: FnOnce(&mut T) -> R,
        R: 'static,
    {
        Some(f(&mut self
            .0
            .as_ref()
            .as_any()
            .downcast_ref::<RefCell<T>>()?
            .borrow_mut()))
    }
}
