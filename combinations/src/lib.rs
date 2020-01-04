// courtesy of c-cube
fn prod(x: Vec<usize>, y: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let x0 = x[0];
    y.iter()
        .map(|y0| {
            let mut y = vec![x0];
            y.extend(y0);
            y
        })
        .collect()
}

pub fn combinations(lst: Vec<usize>, length: usize) -> Vec<Vec<usize>> {
    match length {
        l if l > lst.len() || l == 0 => vec![vec![]],
        l if l == lst.len() => vec![lst],
        1 => lst.iter().map(|x| vec![*x]).collect::<Vec<Vec<usize>>>(),
        _ => vec![
            prod(
                vec![lst[0]],
                combinations(
                    lst.clone().into_iter().skip(1).collect::<Vec<usize>>(),
                    length - 1,
                ),
            ),
            combinations(
                lst.clone().into_iter().skip(1).collect::<Vec<usize>>(),
                length,
            ),
        ]
        .concat(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combinations() {
        assert_eq!(
            vec![
                vec![0, 1, 2],
                vec![0, 1, 3],
                vec![0, 1, 4],
                vec![0, 2, 3],
                vec![0, 2, 4],
                vec![0, 3, 4],
                vec![1, 2, 3],
                vec![1, 2, 4],
                vec![1, 3, 4],
                vec![2, 3, 4],
            ],
            combinations(vec![0, 1, 2, 3, 4], 3),
        )
    }
}
