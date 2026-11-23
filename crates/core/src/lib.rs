//! Pure rules for modular-sofa layouts.
//!
//! Build a [`Draft`], turn it into a [`Layout`] with [`Layout::new`] (which refuses
//! anything that can't be built), then ask for its [`bom`] and [`footprint`].
//! No I/O, no clock, no dependencies: every surface is an adapter on this crate.
//!
//! Lengths are integer tenths of an inch ([`Len`]), so geometry is exact.
//! The side-fit rule is derived and unverified; see [`layout`].

pub mod geom;
pub mod layout;
pub mod piece;
pub mod report;

pub use geom::{Dir, Len, Point, Rect};
pub use layout::{Draft, Layout, LayoutError, PieceRef, PlacedSide};
pub use piece::{Edge, Seat, SeatDepth, SeatId, SideKind, Sides};
pub use report::{Bom, Footprint, bom, footprint};
