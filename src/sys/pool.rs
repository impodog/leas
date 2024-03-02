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

    pub fn visit<T, F, R>(&self, f: F) -> R
    where
        T: Sized + 'static,
        F: FnOnce(Ref<T>) -> R,
        R: 'static,
    {
        f(self
            .0
            .as_ref()
            .as_any()
            .downcast_ref::<RefCell<T>>()
            .unwrap()
            .borrow())
    }

    pub fn visit_mut<T, F, R>(&self, f: F) -> R
    where
        T: Sized + 'static,
        F: FnOnce(RefMut<T>) -> R,
        R: 'static,
    {
        f(self
            .0
            .as_ref()
            .as_any()
            .downcast_ref::<RefCell<T>>()
            .unwrap()
            .borrow_mut())
    }
}
