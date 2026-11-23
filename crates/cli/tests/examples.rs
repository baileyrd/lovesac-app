//! Golden runs over the example files, through the library and the real binary.
#![allow(clippy::unwrap_used)] // test helpers; clippy.toml only covers #[test] fns

use std::path::PathBuf;
use std::process::Command;

use planner_cli::{Outcome, check};

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name)
}

fn check_file(name: &str) -> Outcome {
    check(&std::fs::read_to_string(example(name)).unwrap()).unwrap()
}

#[test]
fn nano_order_golden() {
    let expected = "Layout OK: 7 seats, 9 sides\n\
                    Seats: 4 deep, 3 standard\n\
                    Sides: 7 standard, 2 deep\n\
                    Size:  128 x 117 in (10 ft 8 in x 9 ft 9 in)\n";
    assert_eq!(
        check_file("nano-order.json"),
        Outcome::Valid(expected.into())
    );
}

#[test]
fn four_seats_five_sides_golden() {
    let expected = "Layout OK: 4 seats, 5 sides\n\
                    Seats: 0 deep, 4 standard\n\
                    Sides: 5 standard, 0 deep\n\
                    Size:  117 x 64 in (9 ft 9 in x 5 ft 4 in)\n";
    assert_eq!(
        check_file("4-seats-5-sides.json"),
        Outcome::Valid(expected.into())
    );
}

#[test]
fn refused_golden() {
    let expected = "Layout refused (3 problems):\n\
                    - seat 1's back side spans 41 in; sides come in 35 and 29 in\n\
                    - seat 2's left side spans 29 in, which needs a deep side, and deep sides can't be arms\n\
                    - seat 2 isn't joined edge to edge to the rest\n";
    assert_eq!(
        check_file("refused.json"),
        Outcome::Refused(expected.into())
    );
}

fn exit_code(args: &[&str]) -> Option<i32> {
    Command::new(env!("CARGO_BIN_EXE_planner"))
        .args(args)
        .output()
        .unwrap()
        .status
        .code()
}

#[test]
fn binary_exit_codes() {
    let path = |n: &str| example(n).to_string_lossy().into_owned();
    assert_eq!(exit_code(&["check", &path("nano-order.json")]), Some(0));
    assert_eq!(exit_code(&["check", &path("refused.json")]), Some(1));
    assert_eq!(exit_code(&["check", "no-such-file.json"]), Some(2));
    assert_eq!(exit_code(&["frobnicate", "x"]), Some(2));
    assert_eq!(exit_code(&[]), Some(2));
}
