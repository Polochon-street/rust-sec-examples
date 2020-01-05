fn ethiopian_multiplication(mut x: i32, mut y: i32) -> i32 {
    let halve = |i| -> i32 { i / 2 };
    let double = |i| -> i32 { i * 2 };
    let is_even = |i| -> bool { i % 2 == 0 };

    let mut sum = 0;
    while x >= 1 {
        if !is_even(x) {
            sum += y
        };
        x = halve(x);
        y = double(y);
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethiopian_multiplication() {
        assert_eq!(17 * 34, ethiopian_multiplication(34, 17));
    }
}
