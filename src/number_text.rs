//! Numbers as text that can be read at a glance.

/// A count shortened to K, M, B or T, with one decimal below ten:
/// `910000` is `910K`, `1234567` is `1.2M`. Below a thousand it is exact.
pub fn compact(number: u64) -> String {
    const UNITS: [(u128, &str); 4] = [
        (1_000, "K"),
        (1_000_000, "M"),
        (1_000_000_000, "B"),
        (1_000_000_000_000, "T"),
    ];
    let number = u128::from(number);
    if number < 1_000 {
        return number.to_string();
    }
    for (index, (unit, suffix)) in UNITS.iter().enumerate() {
        let tenths = (number * 10 + unit / 2) / unit;
        if tenths < 100 {
            return match tenths % 10 {
                0 => format!("{}{suffix}", tenths / 10),
                tenth => format!("{}.{tenth}{suffix}", tenths / 10),
            };
        }
        // Rounding up to 1000 of a unit moves on to the next one: `1M`, not `1000K`.
        let whole = (number + unit / 2) / unit;
        if whole < 1_000 || index == UNITS.len() - 1 {
            return format!("{whole}{suffix}");
        }
    }
    unreachable!("the last unit always returns")
}

/// Commas between each group of three digits of `digits`, which must be
/// ASCII digits only: `1234567` is `1,234,567`.
pub fn group_digits(digits: &str) -> String {
    let mut text = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            text.push(',');
        }
        text.push(digit);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_are_shortened_with_one_decimal_below_ten() {
        assert_eq!(compact(0), "0");
        assert_eq!(compact(999), "999");
        assert_eq!(compact(1_000), "1K");
        assert_eq!(compact(1_250), "1.3K");
        assert_eq!(compact(59_000), "59K");
        assert_eq!(compact(910_000), "910K");
        assert_eq!(compact(999_499), "999K");
        assert_eq!(compact(999_500), "1M");
        assert_eq!(compact(1_234_567), "1.2M");
        assert_eq!(compact(2_110_000_000), "2.1B");
        assert_eq!(compact(u64::MAX), "18446744T");
    }

    #[test]
    fn digits_are_grouped_in_threes() {
        assert_eq!(group_digits("0"), "0");
        assert_eq!(group_digits("999"), "999");
        assert_eq!(group_digits("1000"), "1,000");
        assert_eq!(group_digits("1234567"), "1,234,567");
    }
}
