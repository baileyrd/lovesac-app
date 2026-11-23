//! Acceptance cases from the spec's first-slice checklist.

use planner_core::{
    Bom, Dir, Draft, Edge, Layout, LayoutError, Len, PieceRef, Point, Seat, SeatDepth, SeatId,
    Sides, bom, footprint,
};

const STD: SeatDepth = SeatDepth::Standard;
const DEEP: SeatDepth = SeatDepth::Deep;

fn seat(id: u16, x: i32, y: i32, facing: Dir, depth: SeatDepth, edges: &[Edge]) -> Seat {
    Seat {
        id: SeatId(id),
        at: Point {
            x: Len::inches(x),
            y: Len::inches(y),
        },
        facing,
        depth,
        sides: Sides {
            back: edges.contains(&Edge::Back),
            left: edges.contains(&Edge::Left),
            right: edges.contains(&Edge::Right),
        },
    }
}

fn build(seats: Vec<Seat>) -> Result<Layout, Vec<LayoutError>> {
    Layout::new(Draft { seats })
}

fn size(layout: &Layout) -> (Len, Len) {
    let f = footprint(layout);
    (f.width(), f.depth())
}

fn inches(w: i32, d: i32) -> (Len, Len) {
    (Len::inches(w), Len::inches(d))
}

/// Nano's order: a run of 4 deep seats facing south (corner at the west end),
/// 2 standard seats down the west leg, 1 loose standard seat as a chaise.
fn nano_order() -> Vec<Seat> {
    use Edge::*;
    vec![
        seat(1, 6, 6, Dir::S, DEEP, &[Back, Right]),
        seat(2, 35, 6, Dir::S, DEEP, &[Back]),
        seat(3, 64, 6, Dir::S, DEEP, &[Back]),
        seat(4, 93, 6, Dir::S, DEEP, &[Back, Left]),
        seat(5, 6, 41, Dir::E, STD, &[Back]),
        seat(6, 6, 76, Dir::E, STD, &[Back, Right]),
        seat(7, 93, 41, Dir::W, STD, &[]),
    ]
}

#[test]
fn nano_order_reproduces_the_order_sheet() {
    let layout = build(nano_order()).unwrap();
    let expected = Bom {
        standard_seats: 3,
        deep_seats: 4,
        standard_sides: 7,
        deep_sides: 2,
    };
    assert_eq!(bom(&layout), expected);
    assert_eq!(size(&layout), inches(128, 117));
}

#[test]
fn published_4_seats_5_sides() {
    use Edge::*;
    let layout = build(vec![
        seat(1, 6, 6, Dir::S, STD, &[Back, Right]),
        seat(2, 41, 6, Dir::S, STD, &[Back]),
        seat(3, 76, 6, Dir::S, STD, &[Back, Left]),
        seat(4, 6, 35, Dir::S, STD, &[]),
    ])
    .unwrap();
    let expected = Bom {
        standard_seats: 4,
        deep_seats: 0,
        standard_sides: 5,
        deep_sides: 0,
    };
    assert_eq!(bom(&layout), expected);
    assert_eq!(size(&layout), inches(117, 64));
}

#[test]
fn deep_run_takes_two_standard_backs_and_n_minus_2_deep() {
    for n in 2..=10u16 {
        let seats = (0..n)
            .map(|i| {
                let mut edges = vec![Edge::Back];
                if i == 0 {
                    edges.push(Edge::Right);
                }
                if i == n - 1 {
                    edges.push(Edge::Left);
                }
                seat(i + 1, 6 + 29 * i32::from(i), 6, Dir::S, DEEP, &edges)
            })
            .collect();
        let layout = build(seats).unwrap();
        let b = bom(&layout);
        let n = usize::from(n);
        // 2 standard backs + 2 standard arms; the rest of the backs are deep.
        assert_eq!((b.standard_sides, b.deep_sides), (4, n - 2), "n = {n}");
        assert_eq!(size(&layout), inches(29 * n as i32 + 12, 41), "n = {n}");
    }
}

#[test]
fn single_bare_seat_is_a_layout() {
    let layout = build(vec![seat(1, 0, 0, Dir::N, STD, &[])]).unwrap();
    assert_eq!(size(&layout), inches(35, 29));
}

#[test]
fn deep_side_as_arm_is_refused() {
    // A standard seat with an arm and no back: the arm spans only 29 in.
    let err = build(vec![seat(1, 0, 0, Dir::S, STD, &[Edge::Left])]).unwrap_err();
    assert_eq!(
        err,
        [LayoutError::DeepSideAsArm {
            seat: SeatId(1),
            edge: Edge::Left
        }]
    );
}

#[test]
fn span_of_41_is_refused_and_named() {
    // One deep seat with arms at both ends: the back owns both corners, 29 + 6 + 6.
    let err = build(vec![seat(
        3,
        0,
        0,
        Dir::S,
        DEEP,
        &[Edge::Back, Edge::Left, Edge::Right],
    )])
    .unwrap_err();
    let span = LayoutError::UnfitSpan {
        seat: SeatId(3),
        edge: Edge::Back,
        span: Len::inches(41),
    };
    assert_eq!(err, [span]);
    assert_eq!(
        span.to_string(),
        "seat 3's back side spans 41 in; sides come in 35 and 29 in"
    );
}

#[test]
fn overlap_is_refused() {
    let err = build(vec![
        seat(1, 0, 0, Dir::S, STD, &[]),
        seat(2, 10, 0, Dir::S, STD, &[]),
    ])
    .unwrap_err();
    assert!(err.contains(&LayoutError::Overlap(
        PieceRef::Seat(SeatId(1)),
        PieceRef::Seat(SeatId(2))
    )));
}

#[test]
fn side_on_a_covered_edge_overlaps_the_neighbour() {
    let err = build(vec![
        seat(1, 0, 0, Dir::S, STD, &[Edge::Back, Edge::Left]),
        seat(2, 35, 0, Dir::S, STD, &[Edge::Back]),
    ])
    .unwrap_err();
    let overlap = LayoutError::Overlap(
        PieceRef::Seat(SeatId(2)),
        PieceRef::Side(SeatId(1), Edge::Left),
    );
    assert!(err.contains(&overlap), "{err:?}");
}

#[test]
fn gap_is_refused() {
    let err = build(vec![
        seat(1, 0, 0, Dir::S, STD, &[]),
        seat(2, 36, 0, Dir::S, STD, &[]),
    ])
    .unwrap_err();
    assert_eq!(err, [LayoutError::Disconnected(SeatId(2))]);
}

#[test]
fn partial_edge_is_refused() {
    let err = build(vec![
        seat(1, 0, 0, Dir::S, STD, &[]),
        seat(2, 35, 3, Dir::S, STD, &[]),
    ])
    .unwrap_err();
    assert_eq!(err, [LayoutError::Disconnected(SeatId(2))]);
}

#[test]
fn two_islands_are_refused_even_when_each_has_a_neighbour() {
    let err = build(vec![
        seat(1, 0, 0, Dir::S, STD, &[]),
        seat(2, 35, 0, Dir::S, STD, &[]),
        seat(3, 0, 100, Dir::S, STD, &[]),
        seat(4, 35, 100, Dir::S, STD, &[]),
    ])
    .unwrap_err();
    assert_eq!(
        err,
        [
            LayoutError::Disconnected(SeatId(3)),
            LayoutError::Disconnected(SeatId(4))
        ]
    );
}

#[test]
fn empty_and_duplicate_ids_are_refused() {
    assert_eq!(build(vec![]).unwrap_err(), [LayoutError::NoSeats]);
    let err = build(vec![
        seat(1, 0, 0, Dir::S, STD, &[]),
        seat(1, 35, 0, Dir::S, STD, &[]),
    ])
    .unwrap_err();
    assert_eq!(err, [LayoutError::DuplicateSeat(SeatId(1))]);
}

#[test]
fn every_error_is_reported_at_once() {
    let err = build(vec![
        seat(1, 0, 0, Dir::S, STD, &[Edge::Left]),
        seat(
            2,
            100,
            0,
            Dir::S,
            DEEP,
            &[Edge::Back, Edge::Left, Edge::Right],
        ),
    ])
    .unwrap_err();
    assert_eq!(err.len(), 3, "{err:?}");
}
