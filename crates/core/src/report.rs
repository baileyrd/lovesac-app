//! What a valid layout comes to: its parts list and its size.

use crate::geom::{Len, Rect};
use crate::layout::Layout;
use crate::piece::{SeatDepth, SideKind};

/// Bill of materials: pieces counted by type. No prices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Bom {
    /// Seats with the back on the 35 in edge.
    pub standard_seats: usize,
    /// Seats with the back on the 29 in edge.
    pub deep_seats: usize,
    /// 35 in sides.
    pub standard_sides: usize,
    /// 29 in sides.
    pub deep_sides: usize,
}

impl Bom {
    /// Every seat.
    pub fn seats(&self) -> usize {
        self.standard_seats + self.deep_seats
    }

    /// Every side.
    pub fn sides(&self) -> usize {
        self.standard_sides + self.deep_sides
    }
}

/// Counts a layout's pieces.
pub fn bom(layout: &Layout) -> Bom {
    let deep_seats = layout
        .seats()
        .iter()
        .filter(|s| s.depth == SeatDepth::Deep)
        .count();
    let deep_sides = layout
        .sides()
        .iter()
        .filter(|s| s.kind() == SideKind::Deep)
        .count();
    Bom {
        standard_seats: layout.seats().len() - deep_seats,
        deep_seats,
        standard_sides: layout.sides().len() - deep_sides,
        deep_sides,
    }
}

/// How much floor a layout covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Footprint {
    /// The bounding box of every seat and side.
    pub bounds: Rect,
}

impl Footprint {
    /// East–west extent.
    pub fn width(&self) -> Len {
        self.bounds.width()
    }

    /// North–south extent.
    pub fn depth(&self) -> Len {
        self.bounds.depth()
    }
}

/// A layout's footprint. Piece rectangles come from [`Layout::seats`] and [`Layout::sides`].
pub fn footprint(layout: &Layout) -> Footprint {
    Footprint {
        bounds: layout.bounds(),
    }
}
