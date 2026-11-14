//! Personal data: IBANs (mod 97) and payment card numbers (Luhn).

use regex::Regex;
use std::sync::OnceLock;

fn iban_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b[A-Z]{2}[0-9]{2}(?: ?[A-Z0-9]{4}){2,7}(?: ?[A-Z0-9]{1,3})?\b").unwrap()
    })
}

fn card_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(?:[0-9][ -]?){12,18}[0-9]\b").unwrap())
}

fn iban_ok(s: &str) -> bool {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if !(15..=34).contains(&s.len()) {
        return false;
    }
    let rearranged = format!("{}{}", &s[4..], &s[..4]);
    let mut rem: u32 = 0;
    for c in rearranged.chars() {
        let v = match c.to_digit(36) {
            Some(v) => v,
            None => return false,
        };
        // Letters count as two digits (10 to 35).
        rem = if v >= 10 {
            (rem * 100 + v) % 97
        } else {
            (rem * 10 + v) % 97
        };
    }
    rem == 1
}

fn luhn_ok(digits: &str) -> bool {
    let d: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    if !(13..=19).contains(&d.len()) {
        return false;
    }
    let sum: u32 = d
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &x)| {
            if i % 2 == 1 {
                let y = x * 2;
                if y > 9 {
                    y - 9
                } else {
                    y
                }
            } else {
                x
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

/// The 1-based lines that hold a valid IBAN or a valid card number.
pub fn scan(text: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let iban = iban_re().find_iter(line).any(|m| iban_ok(m.as_str()));
        let card = card_re().find_iter(line).any(|m| luhn_ok(m.as_str()));
        if iban || card {
            out.push(i + 1);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_valid_iban_is_found_with_and_without_spaces() {
        assert_eq!(scan("pay to DE89 3704 0044 0532 0130 00 now"), vec![1]);
        assert_eq!(scan("a\nDE89370400440532013000"), vec![2]);
    }

    #[test]
    fn a_wrong_check_digit_is_not_an_iban() {
        assert!(scan("DE88 3704 0044 0532 0130 00").is_empty());
    }

    #[test]
    fn a_valid_card_number_is_found() {
        assert_eq!(scan("card 4111 1111 1111 1111"), vec![1]);
        assert_eq!(scan("card 4111-1111-1111-1111"), vec![1]);
        assert!(scan("card 4111 1111 1111 1112").is_empty());
    }

    #[test]
    fn dates_and_short_numbers_pass() {
        assert!(scan("On 2026-11-14 we had 12345 requests.").is_empty());
    }
}
