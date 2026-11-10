//! Decimal arithmetic and formatting edge cases.
//!
//! Decimals use exact `i128` unscaled values with a retained scale. These tests
//! pin the documented 28-fractional-digit cap and harden the extreme values
//! that used to panic in `abs`/`%`.
use luca_code::types::Dec;

#[test]
fn extreme_unscaled_values_do_not_panic_when_displayed() {
    assert_eq!(
        Dec::new(i128::MIN, 1).display(),
        "-17014118346046923173168730371588410572.8"
    );
    assert_eq!(
        Dec::new(i128::MIN, 0).display(),
        "-170141183460469231731687303715884105728.0"
    );
    assert_eq!(
        Dec::new(i128::MAX, 0).display(),
        "170141183460469231731687303715884105727.0"
    );
}

#[test]
fn extreme_unscaled_values_do_not_panic_in_division_or_remainder() {
    let min = Dec::new(i128::MIN, 0);
    let one = Dec::new(1, 0);
    assert_eq!(
        min.div(&one).unwrap().display(),
        "-170141183460469231731687303715884105728.0"
    );
    // i128::MIN % -1 overflows; it must be reported as an out-of-range result.
    assert!(min.rem(&Dec::new(-1, 0)).is_none());
}

#[test]
fn retained_scale_matches_documented_examples() {
    let a = Dec::parse_literal("20.00").unwrap();
    let b = Dec::parse_literal("1.50").unwrap();
    assert_eq!(a.add(&b).unwrap().display(), "21.50");

    let c = Dec::parse_literal("2.50").unwrap();
    let d = Dec::parse_literal("2.00").unwrap();
    assert_eq!(c.mul(&d).unwrap().display(), "5.00");
}

#[test]
fn non_terminating_division_truncates_to_28_fractional_digits() {
    let quotient = Dec::new(1, 0).div(&Dec::new(3, 0)).unwrap();
    assert_eq!(quotient.scale, 28);
    assert_eq!(quotient.display(), "0.3333333333333333333333333333");
}

#[test]
fn literal_scale_above_28_is_rejected() {
    let too_many = format!("0.{}", "1".repeat(29));
    assert!(Dec::parse_literal(&too_many).is_err());
}
