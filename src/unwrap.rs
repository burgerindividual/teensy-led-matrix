#[inline(always)]
pub fn unwrap<T>(option: Option<T>) -> T {
    if let Some(value) = option {
        value
    } else {
        panic!();
    }
}
