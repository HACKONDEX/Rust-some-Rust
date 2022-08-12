#![forbid(unsafe_code)]

pub fn combinations(arr: &[i32], k: usize) -> Vec<Vec<i32>> {
    let mut k_permutations: Vec<Vec<i32>> = Vec::new();
    let mut cur_perm: Vec<i32> = Vec::with_capacity(k);
    get_next(&mut k_permutations, &mut cur_perm, arr, 0, k, 0);
    k_permutations
}

pub fn get_next(
    permutations: &mut Vec<Vec<i32>>,
    result: &mut Vec<i32>,
    arr: &[i32],
    depth: usize,
    k: usize,
    prev_index: usize,
) {
    if depth == k {
        if result.len() == k {
            permutations.push(result.to_vec());
        }
        return;
    }

    let start = if depth == 0 {
        prev_index
    } else {
        prev_index + 1
    };

    for i in start..arr.len() {
        result.push(arr[i]);
        get_next(permutations, result, arr, depth + 1, k, i);
        result.pop();
    }
}
