#![forbid(unsafe_code)]

pub fn add(x: i32, y: i32) -> i32 {
    x.saturating_add(y)
}
