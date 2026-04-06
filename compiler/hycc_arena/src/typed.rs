use std::cell::UnsafeCell;

#[derive(Debug)]
pub struct TypedArena<T> {
    pub data: UnsafeCell<Vec<Box<T>>>,
}

impl<T> TypedArena<T> {
    pub fn new() -> Self {
        Self {
            data: UnsafeCell::new(Vec::new()),
        }
    }

    pub fn alloc(&self, val: T) -> &T {
        let data = unsafe { &mut *self.data.get() };
        data.push(Box::new(val));

        unsafe { &*(data.last().unwrap().as_ref() as *const T) }
    }
}
