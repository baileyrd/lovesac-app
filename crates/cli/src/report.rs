//! Plain-text rendering of a check. Pure: strings in, strings out.

use planner_core::{Bom, Footprint, LayoutError, Len};

/// The report for a layout that can be built.
pub fn valid(bom: &Bom, fp: &Footprint) -> String {
    format!(
        "Layout OK: {} seats, {} sides\n\
         Seats: {} deep, {} standard\n\
         Sides: {} standard, {} deep\n\
         Size:  {} x {} in ({} x {})\n",
        bom.seats(),
        bom.sides(),
        bom.deep_seats,
        bom.standard_seats,
        bom.standard_sides,
        bom.deep_sides,
        fp.width(),
        fp.depth(),
        feet_inches(fp.width()),
        feet_inches(fp.depth()),
    )
}

/// The report for a refused layout: every problem, one per line.
pub fn refused(errors: &[LayoutError]) -> String {
    let noun = if errors.len() == 1 {
        "problem"
    } else {
        "problems"
    };
    let lines: String = errors.iter().map(|e| format!("- {e}\n")).collect();
    format!("Layout refused ({} {noun}):\n{lines}", errors.len())
}

/// `128` in as `10 ft 8 in`.
fn feet_inches(len: Len) -> String {
    let foot = Len::inches(12).as_tenths();
    let (ft, rest) = (len.as_tenths() / foot, Len::tenths(len.as_tenths() % foot));
    format!("{ft} ft {rest} in")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feet_and_inches() {
        assert_eq!(feet_inches(Len::inches(128)), "10 ft 8 in");
        assert_eq!(feet_inches(Len::inches(36)), "3 ft 0 in");
        assert_eq!(feet_inches(Len::tenths(1175)), "9 ft 9.5 in");
    }

    #[test]
    fn one_problem_is_singular() {
        assert_eq!(
            refused(&[LayoutError::NoSeats]),
            "Layout refused (1 problem):\n- the layout has no seats\n"
        );
    }
}
