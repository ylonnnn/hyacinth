pub fn tag_of<U: Copy, T>(val: &T) -> U {
    unsafe { *(val as *const T as *const U) }
}
