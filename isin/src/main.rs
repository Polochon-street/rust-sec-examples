extern crate luhn_cc;

use luhn_cc::compute_luhn;

fn main() {
    assert_eq!(validate_isin("US0378331005"), true);
    assert_eq!(validate_isin("US0373831005"), false);
    assert_eq!(validate_isin("U50378331005"), false);
    assert_eq!(validate_isin("US03378331005"), false);
    assert_eq!(validate_isin("AU0000XVGZA3"), true);
    assert_eq!(validate_isin("AU0000VXGZA3"), true);
    assert_eq!(validate_isin("FR0000988040"), true);
}

fn validate_isin(isin: &str) -> bool {
    if !isin.chars().all(|x| x.is_alphanumeric()) || isin.len() != 12 {
        return false;
    }
    if !isin[..2].chars().all(|x| x.is_alphabetic())
        || !isin[2..12].chars().all(|x| x.is_alphanumeric())
        || !isin.chars().last().unwrap().is_numeric()
    {
        return false;
    }

    let bytes = isin.as_bytes();

    let first_letter = 'A' as u8;
    let mut string = String::from("");

    for c in bytes {
        let mut temp_str = String::from("");
        if c.is_ascii_digit() {
            temp_str = (*c as char).to_string();
        } else {
            temp_str = (c + 10 - first_letter).to_string();
        }
        string.push_str(&temp_str);
    }
    let number = string.parse::<usize>().unwrap();
    return compute_luhn(number);
}
