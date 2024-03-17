use super::*;

#[derive(Debug, Clone)]
pub struct Resource(Rc<dyn Res>);

#[derive(Clone)]
pub struct WeakResource(Weak<dyn Res>);

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

    pub fn downgrade(&self) -> WeakResource {
        WeakResource(Rc::downgrade(&self.0))
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

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl WeakResource {
    pub fn upgrade(&self) -> Option<Resource> {
        Some(Resource(self.0.upgrade()?))
    }
}

impl fmt::Debug for WeakResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(res) = self.upgrade() {
            write!(f, "{:?}", res)
        } else {
            write!(f, "WeakResource(Deleted)")
        }
    }
}

impl PartialEq for WeakResource {
    fn eq(&self, other: &Self) -> bool {
        if let Some(this) = self.0.upgrade() {
            if let Some(other) = other.0.upgrade() {
                Rc::ptr_eq(&this, &other)
            } else {
                false
            }
        } else {
            false
        }
    }
}
