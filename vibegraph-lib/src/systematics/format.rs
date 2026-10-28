//! The two number spellings MadGraph's systematics output carries that no
//! other writer here produces: Python's `str(float)`, which MadGraph's
//! `write_banner` interpolates into every weight's attributes and description,
//! and Fortran's `Ew.d` edit descriptor, which `unwgt.f` writes the `<mgrwt>`
//! record in.

/// The shortest decimal digits that read back as `value`, and the decimal
/// exponent of the first of them: `value = 0.d₁d₂…dₙ × 10^decpt`. `value` is
/// finite and nonzero.
fn shortest_digits(value: f64) -> (String, i32) {
    // `{:e}` writes the shortest round-tripping digits, `d.ddde±x`.
    let text = format!("{:e}", value.abs());
    let (mantissa, exponent) = text
        .split_once('e')
        .expect("an exponential spelling carries an exponent");
    let exponent: i32 = exponent.parse().expect("the exponent is an integer");
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    (digits.to_string(), exponent + 1)
}

/// Python's `str(value)` (and `repr`) for a float: the shortest digits that
/// read back as `value`, positional with at least one decimal when the
/// leading digit's position `decpt` satisfies `-4 < decpt ≤ 16`, otherwise
/// `d.ddde±XX` with at least two exponent digits and no `.0` added
/// (CPython's `float_repr_style = 'short'`, `format_float_short`).
pub fn py_float(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "inf" } else { "-inf" }.to_string();
    }
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if value == 0.0 {
        return format!("{sign}0.0");
    }
    let (digits, decpt) = shortest_digits(value);
    let n = digits.len() as i32;
    let body = if decpt <= -4 || decpt > 16 {
        let exponent = decpt - 1;
        let mantissa = if n == 1 {
            digits.clone()
        } else {
            format!("{}.{}", &digits[..1], &digits[1..])
        };
        let esign = if exponent < 0 { '-' } else { '+' };
        format!("{mantissa}e{esign}{:02}", exponent.abs())
    } else if decpt <= 0 {
        format!("0.{}{digits}", "0".repeat((-decpt) as usize))
    } else if decpt >= n {
        format!("{digits}{}.0", "0".repeat((decpt - n) as usize))
    } else {
        let split = decpt as usize;
        format!("{}.{}", &digits[..split], &digits[split..])
    };
    format!("{sign}{body}")
}

/// Fortran's `Ew.d` edit descriptor as gfortran writes it: `d` significant
/// digits after `0.`, an exponent `E±XX`, right-justified in `w` columns
/// (` 0.88513229E+02` for `E15.8`). An exponent of three digits drops the
/// `E` (`0.10000000+101`), as the standard prescribes for `Ew.d`.
pub fn fortran_e(value: f64, width: usize, digits: usize) -> String {
    let body = if !value.is_finite() {
        value.to_string()
    } else if value == 0.0 {
        let sign = if value.is_sign_negative() { "-" } else { "" };
        format!("{sign}0.{}E+00", "0".repeat(digits))
    } else {
        let precision = digits - 1;
        let text = format!("{:.precision$e}", value.abs());
        let (mantissa, exponent) = text
            .split_once('e')
            .expect("an exponential spelling carries an exponent");
        let exponent: i32 = exponent.parse::<i32>().expect("integer exponent") + 1;
        let figures: String = mantissa.chars().filter(char::is_ascii_digit).collect();
        let sign = if value < 0.0 { "-" } else { "" };
        let esign = if exponent < 0 { '-' } else { '+' };
        if exponent.abs() <= 99 {
            format!("{sign}0.{figures}E{esign}{:02}", exponent.abs())
        } else {
            format!("{sign}0.{figures}{esign}{:03}", exponent.abs())
        }
    };
    format!("{body:>width$}")
}

/// C's `%+.7e`, the spelling of a `<wgt>` value (`lhe_parser.py:2620`):
/// eight significant digits, a sign always, and an exponent of at least two
/// digits with its sign.
pub fn wgt_text(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let text = format!("{value:+.7e}");
    let (mantissa, exponent) = text
        .split_once('e')
        .expect("an exponential spelling carries an exponent");
    let exponent: i32 = exponent.parse().expect("integer exponent");
    let sign = if exponent < 0 { '-' } else { '+' };
    format!("{mantissa}e{sign}{:02}", exponent.abs())
}

/// Fortran's `Iw`: the integer right-justified in `w` columns.
pub fn fortran_i(value: i64, width: usize) -> String {
    format!("{value:>width$}")
}

/// The value a number written as `fortran_e(value, 15, 8)` reads back as:
/// what a reader of the written record, MadGraph's module among them, sees.
pub fn as_written_e15_8(value: f64) -> f64 {
    fortran_e(value, 15, 8)
        .trim()
        .parse()
        .expect("an E15.8 spelling of a finite value parses")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The spellings CPython 3 gives `str(x)` for each value, as printed by
    /// `python3 -c 'for x in [...]: print(str(x))'`: the table the design pins
    /// the header's numbers against.
    #[test]
    fn py_float_matches_cpythons_str() {
        for (value, python) in [
            (0.5, "0.5"),
            (1.0, "1.0"),
            (2.0, "2.0"),
            (0.25, "0.25"),
            (1.0 / 3.0, "0.3333333333333333"),
            (1e-05, "1e-05"),
            (0.0001, "0.0001"),
            (1000000000000000.0, "1000000000000000.0"),
            (1e16, "1e+16"),
            (1.5e16, "1.5e+16"),
            (1.2345678901234568e17, "1.2345678901234568e+17"),
            (0.1 + 0.2, "0.30000000000000004"),
            (-0.5, "-0.5"),
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (123.456, "123.456"),
            (1e100, "1e+100"),
            (2.5e-300, "2.5e-300"),
            (12345678901234567.0, "1.2345678901234568e+16"),
            (100.0, "100.0"),
        ] {
            assert_eq!(py_float(value), python, "{value:e}");
        }
    }

    #[test]
    fn fortran_e_matches_gfortran() {
        assert_eq!(fortran_e(88.513229, 15, 8), " 0.88513229E+02");
        assert_eq!(fortran_e(0.0011720939, 15, 8), " 0.11720939E-02");
        assert_eq!(fortran_e(-88.513229, 15, 8), "-0.88513229E+02");
        assert_eq!(fortran_e(0.0, 15, 8), " 0.00000000E+00");
        assert_eq!(fortran_e(7891.6645, 15, 8), " 0.78916645E+04");
        assert_eq!(fortran_e(1.0, 15, 8), " 0.10000000E+01");
        // Rounding carries into the exponent.
        assert_eq!(fortran_e(9.999999999, 15, 8), " 0.10000000E+02");
        assert_eq!(fortran_e(1e100, 15, 8), " 0.10000000+101");
        assert_eq!(fortran_i(2, 3), "  2");
        assert_eq!(fortran_i(-11, 9), "      -11");
        assert_eq!(as_written_e15_8(88.5132286959), 88.513229);
        assert_eq!(wgt_text(1488.9092), "+1.4889092e+03");
        assert_eq!(wgt_text(-2.0e-12), "-2.0000000e-12");
    }
}
