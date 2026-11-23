//! Exact plan-view geometry in tenths of an inch.
//!
//! Plan coordinates: `x` grows east, `y` grows south (screen order), so north is up.

use std::fmt;
use std::ops::{Add, Sub};

/// A length in tenths of an inch. Integer so adjacency and equality are exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Len(i32);

impl Len {
    /// Zero length.
    pub const ZERO: Len = Len(0);

    /// A length of `t` tenths of an inch.
    pub const fn tenths(t: i32) -> Self {
        Len(t)
    }

    /// A length of `i` whole inches.
    pub const fn inches(i: i32) -> Self {
        Len(i * 10)
    }

    /// The length in tenths of an inch.
    pub const fn as_tenths(self) -> i32 {
        self.0
    }
}

impl Add for Len {
    type Output = Len;
    fn add(self, rhs: Len) -> Len {
        Len(self.0 + rhs.0)
    }
}

impl Sub for Len {
    type Output = Len;
    fn sub(self, rhs: Len) -> Len {
        Len(self.0 - rhs.0)
    }
}

/// Inches, with one decimal only when needed: `128`, `28.5`.
impl fmt::Display for Len {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        match abs % 10 {
            0 => write!(f, "{sign}{}", abs / 10),
            frac => write!(f, "{sign}{}.{frac}", abs / 10),
        }
    }
}

/// A point on the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    /// Distance east of the origin.
    pub x: Len,
    /// Distance south of the origin.
    pub y: Len,
}

/// A compass direction on the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    /// Up the plan (−y).
    N,
    /// Right (+x).
    E,
    /// Down the plan (+y).
    S,
    /// Left (−x).
    W,
}

impl Dir {
    /// The direction behind this one.
    pub const fn opposite(self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::E => Dir::W,
            Dir::S => Dir::N,
            Dir::W => Dir::E,
        }
    }

    /// A quarter turn clockwise: a sitter facing `self` has their right hand this way.
    pub const fn cw(self) -> Dir {
        match self {
            Dir::N => Dir::E,
            Dir::E => Dir::S,
            Dir::S => Dir::W,
            Dir::W => Dir::N,
        }
    }

    /// A quarter turn counter-clockwise: a sitter facing `self` has their left hand this way.
    pub const fn ccw(self) -> Dir {
        self.cw().opposite()
    }

    /// True for `N` and `S`.
    pub const fn is_vertical(self) -> bool {
        matches!(self, Dir::N | Dir::S)
    }
}

/// An axis-aligned rectangle, half-open: `[x0, x1) × [y0, y1)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    x0: Len,
    y0: Len,
    x1: Len,
    y1: Len,
}

impl Rect {
    /// The rectangle with its north-west corner at `at`, `w` wide (x) and `h` tall (y).
    pub fn new(at: Point, w: Len, h: Len) -> Rect {
        Rect {
            x0: at.x,
            y0: at.y,
            x1: at.x + w,
            y1: at.y + h,
        }
    }

    /// Extent along x.
    pub fn width(&self) -> Len {
        self.x1 - self.x0
    }

    /// Extent along y.
    pub fn depth(&self) -> Len {
        self.y1 - self.y0
    }

    /// The north-west corner.
    pub fn min(&self) -> Point {
        Point {
            x: self.x0,
            y: self.y0,
        }
    }

    /// True when the interiors intersect. Touching edges do not overlap.
    pub fn overlaps(&self, o: &Rect) -> bool {
        self.x0 < o.x1 && o.x0 < self.x1 && self.y0 < o.y1 && o.y0 < self.y1
    }

    /// True when the two share one whole edge exactly (same length, same ends).
    pub fn shares_edge(&self, o: &Rect) -> bool {
        let same_rows = self.y0 == o.y0 && self.y1 == o.y1;
        let same_cols = self.x0 == o.x0 && self.x1 == o.x1;
        let side_by_side = self.x1 == o.x0 || o.x1 == self.x0;
        let stacked = self.y1 == o.y0 || o.y1 == self.y0;
        (same_rows && side_by_side) || (same_cols && stacked)
    }

    /// The smallest rectangle holding both.
    pub fn union(&self, o: &Rect) -> Rect {
        Rect {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }

    /// The strip `thick` deep just outside the edge facing `dir`.
    pub fn strip(&self, dir: Dir, thick: Len) -> Rect {
        let Rect { x0, y0, x1, y1 } = *self;
        match dir {
            Dir::N => Rect {
                y0: y0 - thick,
                y1: y0,
                ..*self
            },
            Dir::S => Rect {
                y0: y1,
                y1: y1 + thick,
                ..*self
            },
            Dir::W => Rect {
                x0: x0 - thick,
                x1: x0,
                ..*self
            },
            Dir::E => Rect {
                x0: x1,
                x1: x1 + thick,
                ..*self
            },
        }
    }

    /// This rectangle stretched by `by` toward `dir`.
    pub fn grow(&self, dir: Dir, by: Len) -> Rect {
        match dir {
            Dir::N => Rect {
                y0: self.y0 - by,
                ..*self
            },
            Dir::S => Rect {
                y1: self.y1 + by,
                ..*self
            },
            Dir::W => Rect {
                x0: self.x0 - by,
                ..*self
            },
            Dir::E => Rect {
                x1: self.x1 + by,
                ..*self
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect::new(
            Point {
                x: Len::inches(x),
                y: Len::inches(y),
            },
            Len::inches(w),
            Len::inches(h),
        )
    }

    #[test]
    fn len_displays_inches() {
        assert_eq!(Len::inches(128).to_string(), "128");
        assert_eq!(Len::tenths(285).to_string(), "28.5");
        assert_eq!(Len::tenths(-5).to_string(), "-0.5");
    }

    #[test]
    fn sitter_hands() {
        // Facing south, your left hand points east.
        assert_eq!(Dir::S.ccw(), Dir::E);
        assert_eq!(Dir::S.cw(), Dir::W);
        assert_eq!(Dir::E.ccw(), Dir::N);
        assert_eq!(Dir::N.cw().cw(), Dir::S);
    }

    #[test]
    fn touching_is_not_overlap() {
        assert!(!r(0, 0, 10, 10).overlaps(&r(10, 0, 10, 10)));
        assert!(r(0, 0, 10, 10).overlaps(&r(9, 9, 10, 10)));
    }

    #[test]
    fn shared_edge_must_match_whole_edge() {
        assert!(r(0, 0, 29, 35).shares_edge(&r(29, 0, 29, 35)));
        assert!(r(0, 0, 29, 35).shares_edge(&r(0, 35, 29, 35)));
        assert!(!r(0, 0, 29, 35).shares_edge(&r(29, 1, 29, 35)), "offset");
        assert!(!r(0, 0, 29, 35).shares_edge(&r(30, 0, 29, 35)), "gap");
        assert!(!r(0, 0, 35, 29).shares_edge(&r(0, 29, 29, 35)), "unequal");
    }

    #[test]
    fn strip_and_grow() {
        let seat = r(10, 10, 35, 29);
        assert_eq!(seat.strip(Dir::N, Len::inches(6)), r(10, 4, 35, 6));
        assert_eq!(seat.strip(Dir::E, Len::inches(6)), r(45, 10, 6, 29));
        assert_eq!(r(10, 4, 35, 6).grow(Dir::W, Len::inches(6)), r(4, 4, 41, 6));
    }
}
