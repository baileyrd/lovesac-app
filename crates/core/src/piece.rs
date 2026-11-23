//! The catalog: one seat in two orientations and two side lengths.
//!
//! Sizes come from the maker's published dimensions (see the research doc).

use std::fmt;

use crate::geom::{Dir, Len, Point, Rect};

/// A seat's long edge (35 in).
pub const SEAT_LONG: Len = Len::inches(35);
/// A seat's short edge (29 in).
pub const SEAT_SHORT: Len = Len::inches(29);
/// How far a side stands out from the seat edge it sits on (6 in).
pub const SIDE_THICK: Len = Len::inches(6);

/// How a seat is turned relative to its back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeatDepth {
    /// Back on the 35 in edge: 35 wide, 29 deep.
    Standard,
    /// The same seat turned 90°: back on the 29 in edge, 29 wide, 35 deep.
    Deep,
}

/// The two side pieces. Kinds are derived from the layout, never declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideKind {
    /// 35 in long; used as a back or an arm.
    Standard,
    /// 29 in long; used as a back only.
    Deep,
}

impl SideKind {
    /// The piece's length.
    pub const fn len(self) -> Len {
        match self {
            SideKind::Standard => SEAT_LONG,
            SideKind::Deep => SEAT_SHORT,
        }
    }
}

/// A seat edge that can carry a side. There is no front: nothing stands where you sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    /// Behind the sitter.
    Back,
    /// On the sitter's left hand.
    Left,
    /// On the sitter's right hand.
    Right,
}

impl Edge {
    /// Every edge, in a fixed order.
    pub const ALL: [Edge; 3] = [Edge::Back, Edge::Left, Edge::Right];

    /// The edges that meet this one at a back corner.
    pub(crate) const fn corner_partners(self) -> &'static [Edge] {
        match self {
            Edge::Back => &[Edge::Left, Edge::Right],
            Edge::Left | Edge::Right => &[Edge::Back],
        }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Edge::Back => "back",
            Edge::Left => "left",
            Edge::Right => "right",
        })
    }
}

/// A seat's identifier, unique within a layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeatId(pub u16);

impl fmt::Display for SeatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "seat {}", self.0)
    }
}

/// Which of a seat's edges carry a side. A set, so a doubled side can't be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Sides {
    /// A side behind the sitter.
    pub back: bool,
    /// A side on the sitter's left.
    pub left: bool,
    /// A side on the sitter's right.
    pub right: bool,
}

impl Sides {
    /// True when `edge` carries a side.
    pub const fn has(self, edge: Edge) -> bool {
        match edge {
            Edge::Back => self.back,
            Edge::Left => self.left,
            Edge::Right => self.right,
        }
    }

    /// The edges that carry a side, in [`Edge::ALL`] order.
    pub fn edges(self) -> impl Iterator<Item = Edge> {
        Edge::ALL.into_iter().filter(move |&e| self.has(e))
    }
}

/// A seat placed on the plan, with the sides on its edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seat {
    /// Identifier, unique within a layout.
    pub id: SeatId,
    /// North-west corner of the seat's own rectangle (sides excluded).
    pub at: Point,
    /// The way a sitter faces; the back is opposite.
    pub facing: Dir,
    /// Which edge the back runs along.
    pub depth: SeatDepth,
    /// The sides standing on this seat's edges.
    pub sides: Sides,
}

impl Seat {
    /// The seat's own rectangle on the plan.
    pub fn rect(&self) -> Rect {
        let (along_back, deep) = (self.edge_len(Edge::Back), self.edge_len(Edge::Left));
        match self.facing.is_vertical() {
            true => Rect::new(self.at, along_back, deep),
            false => Rect::new(self.at, deep, along_back),
        }
    }

    /// The plan direction an edge faces outward.
    pub fn outward(&self, edge: Edge) -> Dir {
        match edge {
            Edge::Back => self.facing.opposite(),
            Edge::Left => self.facing.ccw(),
            Edge::Right => self.facing.cw(),
        }
    }

    /// The length of a seat edge.
    pub fn edge_len(&self, edge: Edge) -> Len {
        match (self.depth, edge) {
            (SeatDepth::Standard, Edge::Back) | (SeatDepth::Deep, Edge::Left | Edge::Right) => {
                SEAT_LONG
            }
            (SeatDepth::Standard, Edge::Left | Edge::Right) | (SeatDepth::Deep, Edge::Back) => {
                SEAT_SHORT
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seat(facing: Dir, depth: SeatDepth) -> Seat {
        Seat {
            id: SeatId(1),
            at: Point {
                x: Len::ZERO,
                y: Len::ZERO,
            },
            facing,
            depth,
            sides: Sides::default(),
        }
    }

    #[test]
    fn standard_facing_south_is_35_wide() {
        let r = seat(Dir::S, SeatDepth::Standard).rect();
        assert_eq!((r.width(), r.depth()), (SEAT_LONG, SEAT_SHORT));
    }

    #[test]
    fn deep_is_the_same_seat_turned() {
        let deep = seat(Dir::S, SeatDepth::Deep).rect();
        let turned = seat(Dir::E, SeatDepth::Standard).rect();
        assert_eq!(deep, turned);
    }

    #[test]
    fn edges_face_outward_from_the_sitter() {
        let s = seat(Dir::S, SeatDepth::Standard);
        assert_eq!(s.outward(Edge::Back), Dir::N);
        assert_eq!(s.outward(Edge::Left), Dir::E);
        assert_eq!(s.outward(Edge::Right), Dir::W);
    }

    #[test]
    fn sides_iterate_in_order() {
        let s = Sides {
            back: true,
            left: false,
            right: true,
        };
        assert_eq!(s.edges().collect::<Vec<_>>(), [Edge::Back, Edge::Right]);
    }
}
