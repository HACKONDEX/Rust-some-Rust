#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;

fn get_lines(file: File) -> HashSet<String> {
    let mut lines_set = HashSet::new();
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        if line.is_err() {
            continue;
        }
        lines_set.insert(line.unwrap());
    }
    lines_set
}

fn print_matching_lines(file: File, mut lines_set: HashSet<String>) {
    let reader = std::io::BufReader::new(file);

    for line in reader.lines() {
        if line.is_err() {
            continue;
        }
        let str: String = line.unwrap();
        if lines_set.contains(&str) {
            lines_set.take(&str);
            println!("{}", str);
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let file1 = std::fs::File::open(&args[1]).unwrap();
    let file2 = std::fs::File::open(&args[2]).unwrap();

    let lines_set = get_lines(file1);
    print_matching_lines(file2, lines_set);
}
