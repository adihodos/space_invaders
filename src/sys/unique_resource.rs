trait ResourceDeleter {
    type Handle;

    fn is_null(res: &Self::Handle) -> bool;
    fn null() -> Self::Handle;
    fn delete(&mut self, res: &mut Self::Handle);
}

struct UniqueResource<T: ResourceDeleter> {
    handle: Option<<T as ResourceDeleter>::Handle>,
    deleter: T,
}

impl<T: ResourceDeleter> UniqueResource<T> {

    fn new() -> Self
    where
        T: std::default::Default,
    {
        Self {
            handle: None,
            deleter: T::default(),
        }
    }

    fn from_handle(handle: <T as ResourceDeleter>::Handle) -> Self
    where
        T: std::default::Default,
    {
        Self {
            handle: if T::is_null(&handle) {
                None
            } else {
                Some(handle)
            },
            deleter: T::default(),
        }
    }

    fn from_state_handle(handle: <T as ResourceDeleter>::Handle, deleter: T) -> Self {
        Self {
            handle: if T::is_null(&handle) {
                None
            } else {
                Some(handle)
            },
            deleter,
        }
    }

    fn handle_ref(&self) -> Option<&<T as ResourceDeleter>::Handle> {
        self.handle.as_ref()
    }

    fn handle_ref_mut(&mut self) -> Option<&mut <T as ResourceDeleter>::Handle> {
        self.handle.as_mut()
    }
}

impl<T: ResourceDeleter> std::ops::Drop for UniqueResource<T> {
    fn drop(&mut self) {
        self.handle.take().as_mut().and_then(|mut handle| {
            self.deleter.delete(&mut handle);
            Some(())
        });
    }
}
