//! Drafts, the structural rules, and the only way to get a [`Layout`].
//!
//! # Side fit (derived, not published: verify against a real set)
//!
//! A side covers its seat edge plus [`SIDE_THICK`] for each back corner it owns.
//! Where a back and an arm meet on one seat, the side on the seat's 29 in edge
//! owns that corner. A span of 35 in takes a standard side; 29 in takes a deep
//! side, and only as a back; any other span is refused.
//!
//! This reproduces the maker's "deep sides for 3+ deep seats in a row" and the
//! research doc's run rule, but it is our derivation from published sizes, not
//! a rule the maker states. In particular it refuses a standard side as an arm
//! on a seat with no back, which is unverified.

use std::collections::HashSet;
use std::fmt;

use crate::geom::{Len, Rect};
use crate::piece::{Edge, SEAT_SHORT, SIDE_THICK, Seat, SeatId, SideKind};

/// What an editor holds while drawing. Anything goes until [`Layout::new`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Draft {
    /// The seats, each with the sides on its edges.
    pub seats: Vec<Seat>,
}

/// A piece named in an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceRef {
    /// A seat.
    Seat(SeatId),
    /// The side on a seat's edge.
    Side(SeatId, Edge),
}

impl fmt::Display for PieceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PieceRef::Seat(id) => write!(f, "{id}"),
            PieceRef::Side(id, edge) => write!(f, "{id}'s {edge} side"),
        }
    }
}

/// Why a draft can't be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutError {
    /// A layout needs at least one seat.
    NoSeats,
    /// Two seats share an id.
    DuplicateSeat(SeatId),
    /// Two pieces occupy the same floor.
    Overlap(PieceRef, PieceRef),
    /// The seat isn't joined, edge to whole edge, to the rest of the layout.
    Disconnected(SeatId),
    /// No side piece is this long.
    UnfitSpan {
        /// Seat carrying the side.
        seat: SeatId,
        /// Edge the side stands on.
        edge: Edge,
        /// The length the side would need.
        span: Len,
    },
    /// Only a deep side would fit here, and deep sides can't be arms.
    DeepSideAsArm {
        /// Seat carrying the side.
        seat: SeatId,
        /// Edge the side stands on.
        edge: Edge,
    },
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutError::NoSeats => f.write_str("the layout has no seats"),
            LayoutError::DuplicateSeat(id) => write!(f, "{id} is used more than once"),
            LayoutError::Overlap(a, b) => write!(f, "{a} overlaps {b}"),
            LayoutError::Disconnected(id) => {
                write!(f, "{id} isn't joined edge to edge to the rest")
            }
            LayoutError::UnfitSpan { seat, edge, span } => write!(
                f,
                "{} spans {span} in; sides come in 35 and 29 in",
                PieceRef::Side(*seat, *edge)
            ),
            LayoutError::DeepSideAsArm { seat, edge } => write!(
                f,
                "{} spans 29 in, which needs a deep side, and deep sides can't be arms",
                PieceRef::Side(*seat, *edge)
            ),
        }
    }
}

impl std::error::Error for LayoutError {}

/// A side in place, with the kind its span needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacedSide {
    seat: SeatId,
    edge: Edge,
    kind: SideKind,
    rect: Rect,
}

impl PlacedSide {
    /// The seat it stands on.
    pub fn seat(&self) -> SeatId {
        self.seat
    }
    /// The edge it stands on.
    pub fn edge(&self) -> Edge {
        self.edge
    }
    /// The piece it takes.
    pub fn kind(&self) -> SideKind {
        self.kind
    }
    /// Its rectangle on the plan.
    pub fn rect(&self) -> Rect {
        self.rect
    }
}

/// A layout that can be built. Only [`Layout::new`] makes one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    seats: Vec<Seat>,
    sides: Vec<PlacedSide>,
    bounds: Rect,
}

impl Layout {
    /// Checks every structural rule and returns all the errors found, not just the first.
    pub fn new(draft: Draft) -> Result<Layout, Vec<LayoutError>> {
        let seats = draft.seats;
        let Some(first) = seats.first() else {
            return Err(vec![LayoutError::NoSeats]);
        };
        let mut errors = duplicate_ids(&seats);
        let mut sides = Vec::new();
        for result in seats.iter().flat_map(place_sides) {
            match result {
                Ok(side) => sides.push(side),
                Err(e) => errors.push(e),
            }
        }
        errors.extend(overlaps(&seats, &sides));
        errors.extend(disconnected(&seats));
        if !errors.is_empty() {
            return Err(errors);
        }
        let bounds = seats
            .iter()
            .map(Seat::rect)
            .chain(sides.iter().map(PlacedSide::rect))
            .fold(first.rect(), |acc, r| acc.union(&r));
        Ok(Layout {
            seats,
            sides,
            bounds,
        })
    }

    /// The seats, in draft order.
    pub fn seats(&self) -> &[Seat] {
        &self.seats
    }

    /// The sides, each with its derived kind.
    pub fn sides(&self) -> &[PlacedSide] {
        &self.sides
    }

    /// The bounding box of every piece.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }
}

fn duplicate_ids(seats: &[Seat]) -> Vec<LayoutError> {
    let mut seen = HashSet::new();
    let mut reported = HashSet::new();
    seats
        .iter()
        .filter(|s| !seen.insert(s.id) && reported.insert(s.id))
        .map(|s| LayoutError::DuplicateSeat(s.id))
        .collect()
}

fn place_sides(seat: &Seat) -> impl Iterator<Item = Result<PlacedSide, LayoutError>> + '_ {
    seat.sides.edges().map(move |edge| place_side(seat, edge))
}

/// Places one side, stretched over the back corners it owns.
fn place_side(seat: &Seat, edge: Edge) -> Result<PlacedSide, LayoutError> {
    let owns_corners = seat.edge_len(edge) == SEAT_SHORT;
    let strip = seat.rect().strip(seat.outward(edge), SIDE_THICK);
    let (span, rect) = edge
        .corner_partners()
        .iter()
        .filter(|&&p| owns_corners && seat.sides.has(p))
        .fold((seat.edge_len(edge), strip), |(span, rect), &p| {
            (span + SIDE_THICK, rect.grow(seat.outward(p), SIDE_THICK))
        });
    let kind = fit(seat.id, edge, span)?;
    Ok(PlacedSide {
        seat: seat.id,
        edge,
        kind,
        rect,
    })
}

fn fit(seat: SeatId, edge: Edge, span: Len) -> Result<SideKind, LayoutError> {
    if span == SideKind::Standard.len() {
        return Ok(SideKind::Standard);
    }
    if span != SideKind::Deep.len() {
        return Err(LayoutError::UnfitSpan { seat, edge, span });
    }
    if edge != Edge::Back {
        return Err(LayoutError::DeepSideAsArm { seat, edge });
    }
    Ok(SideKind::Deep)
}

fn overlaps(seats: &[Seat], sides: &[PlacedSide]) -> Vec<LayoutError> {
    let pieces: Vec<(PieceRef, Rect)> = seats
        .iter()
        .map(|s| (PieceRef::Seat(s.id), s.rect()))
        .chain(
            sides
                .iter()
                .map(|s| (PieceRef::Side(s.seat, s.edge), s.rect)),
        )
        .collect();
    pieces
        .iter()
        .enumerate()
        .flat_map(|(i, a)| pieces[i + 1..].iter().map(move |b| (a, b)))
        .filter(|((_, ra), (_, rb))| ra.overlaps(rb))
        .map(|((a, _), (b, _))| LayoutError::Overlap(*a, *b))
        .collect()
}

/// Seats not reachable from the first one through whole shared edges.
fn disconnected(seats: &[Seat]) -> Vec<LayoutError> {
    let rects: Vec<Rect> = seats.iter().map(Seat::rect).collect();
    let mut reached = vec![false; seats.len()];
    let mut queue = vec![0];
    while let Some(i) = queue.pop() {
        if std::mem::replace(&mut reached[i], true) {
            continue;
        }
        queue.extend((0..rects.len()).filter(|&j| !reached[j] && rects[i].shares_edge(&rects[j])));
    }
    seats
        .iter()
        .zip(reached)
        .filter(|(_, r)| !r)
        .map(|(s, _)| LayoutError::Disconnected(s.id))
        .collect()
}
