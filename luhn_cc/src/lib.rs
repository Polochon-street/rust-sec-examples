#[cfg(test)]
mod tests {
    use super::{compute_luhn, DigitIter};

    #[test]
    fn valid_ccn() {
        assert_eq!(compute_luhn(49927398716), true);
        assert_eq!(compute_luhn(1234567812345670), true);
    }

    #[test]
    fn invalid_ccn() {
        assert_eq!(compute_luhn(49927398717), false);
        assert_eq!(compute_luhn(1234567812345678), false);
    }
}

struct DigitIter(pub usize);

impl Iterator for DigitIter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.0 == 0 {
            None
        }
        else {
            let x = self.0 % 10;
            self.0 = self.0 / 10;
            Some(x)
        }
    }
}

pub fn compute_luhn(number: usize) -> bool {
    let iter_number = DigitIter(number);
    let s = iter_number.enumerate()
        .map(|(i, digit)| {
            if (i % 2) != 0 {
                if (2 * digit) > 9 { 2 * digit - 9 } else { 2 * digit }
            } else { digit }
        })
        .sum::<usize>();

    return s % 10 == 0;
}
