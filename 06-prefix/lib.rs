#![forbid(unsafe_code)]

use std::str::Chars;

pub fn longest_common_prefix(strs: Vec<&str>) -> String {
    match strs.len() {
        0 => String::new(),
        1 => String::from(strs[0]),
        _ => find_longest_common_prefix(strs),
    }
}

pub fn find_longest_common_prefix(strs: Vec<&str>) -> String {
    let mut prefix: String = String::new();
    let mut iters: Vec<Chars> = Vec::new();
    for str in strs {
        iters.push(str.chars());
    }

    'outer: loop {
        let symbol_option = iters[0].next();
        if symbol_option.is_none() {
            break;
        }
        for iter in iters.iter_mut().skip(1) {
            if iter.next() != symbol_option {
                break 'outer;
            }
        }
        prefix.push(symbol_option.unwrap());
    }
    prefix
}
