//! Layout file adapter: JSON text in, a core [`Draft`] out.
//!
//! Format v1, lengths in inches (tenths allowed), `x`/`y` the seat's north-west corner:
//!
//! ```json
//! { "v": 1,
//!   "seats": [ { "id": 1, "x": 6, "y": 6, "facing": "S", "depth": "deep",
//!                "sides": ["back", "right"] } ] }
//! ```
//!
//! Every field is validated here, and unknown keys are refused so typos don't pass silently.

use std::fmt;

use planner_core::{Dir, Draft, Edge, Len, Point, Seat, SeatDepth, SeatId, Sides};
use serde_json::{Map, Value};

/// The only format version this reader accepts.
pub const VERSION: u64 = 1;
/// Largest coordinate accepted, in tenths of an inch (10,000 in).
const MAX_TENTHS: f64 = 100_000.0;

/// Why a layout file can't be read, with the path to the bad field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileError {
    at: String,
    msg: String,
}

impl FileError {
    fn new(at: impl Into<String>, msg: impl Into<String>) -> Self {
        FileError {
            at: at.into(),
            msg: msg.into(),
        }
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.at.is_empty() {
            true => f.write_str(&self.msg),
            false => write!(f, "{} {}", self.at, self.msg),
        }
    }
}

impl std::error::Error for FileError {}

/// Parses a v1 layout file into a draft. Structural rules are left to the core.
pub fn parse(text: &str) -> Result<Draft, FileError> {
    let root: Value = serde_json::from_str(text)
        .map_err(|e| FileError::new("", format!("not valid JSON: {e}")))?;
    let obj = object(&root, "file")?;
    known_keys(obj, "", &["v", "seats"])?;
    if obj.get("v").and_then(Value::as_u64) != Some(VERSION) {
        return Err(FileError::new("v", format!("must be {VERSION}")));
    }
    let seats = obj
        .get("seats")
        .and_then(Value::as_array)
        .ok_or_else(|| FileError::new("seats", "must be a list"))?;
    let seats = seats
        .iter()
        .enumerate()
        .map(|(i, v)| seat(v, &format!("seats[{i}]")))
        .collect::<Result<_, _>>()?;
    Ok(Draft { seats })
}

fn seat(v: &Value, at: &str) -> Result<Seat, FileError> {
    let obj = object(v, at)?;
    known_keys(obj, at, &["id", "x", "y", "facing", "depth", "sides"])?;
    let field = |k: &str| {
        obj.get(k)
            .map(|v| (v, path(at, k)))
            .ok_or_else(|| FileError::new(path(at, k), "is missing"))
    };
    let (id, x, y, facing, depth) = (
        field("id")?,
        field("x")?,
        field("y")?,
        field("facing")?,
        field("depth")?,
    );
    Ok(Seat {
        id: seat_id(id.0, &id.1)?,
        at: Point {
            x: inches(x.0, &x.1)?,
            y: inches(y.0, &y.1)?,
        },
        facing: pick(
            facing.0,
            &facing.1,
            &[("N", Dir::N), ("E", Dir::E), ("S", Dir::S), ("W", Dir::W)],
        )?,
        depth: pick(
            depth.0,
            &depth.1,
            &[("standard", SeatDepth::Standard), ("deep", SeatDepth::Deep)],
        )?,
        sides: sides(obj.get("sides"), &path(at, "sides"))?,
    })
}

fn sides(v: Option<&Value>, at: &str) -> Result<Sides, FileError> {
    let Some(v) = v else {
        return Ok(Sides::default());
    };
    let list = v
        .as_array()
        .ok_or_else(|| FileError::new(at, "must be a list"))?;
    let mut sides = Sides::default();
    for (i, v) in list.iter().enumerate() {
        let at = format!("{at}[{i}]");
        let edge = pick(
            v,
            &at,
            &[
                ("back", Edge::Back),
                ("left", Edge::Left),
                ("right", Edge::Right),
            ],
        )?;
        let slot = match edge {
            Edge::Back => &mut sides.back,
            Edge::Left => &mut sides.left,
            Edge::Right => &mut sides.right,
        };
        if std::mem::replace(slot, true) {
            return Err(FileError::new(at, format!("repeats the {edge} side")));
        }
    }
    Ok(sides)
}

fn seat_id(v: &Value, at: &str) -> Result<SeatId, FileError> {
    v.as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .map(SeatId)
        .ok_or_else(|| FileError::new(at, "must be a whole number from 0 to 65535"))
}

/// Inches as a number with at most one decimal place.
fn inches(v: &Value, at: &str) -> Result<Len, FileError> {
    let n = v
        .as_f64()
        .ok_or_else(|| FileError::new(at, "must be a number of inches"))?;
    let tenths = (n * 10.0).round();
    if (n * 10.0 - tenths).abs() > 1e-6 {
        return Err(FileError::new(at, "must be in whole tenths of an inch"));
    }
    if tenths.abs() > MAX_TENTHS {
        return Err(FileError::new(at, "is out of range (±10000 in)"));
    }
    // In range by the check above, so the cast is exact.
    Ok(Len::tenths(tenths as i32))
}

fn pick<T: Copy>(v: &Value, at: &str, options: &[(&str, T)]) -> Result<T, FileError> {
    let s = v.as_str().unwrap_or_default();
    options
        .iter()
        .find(|(name, _)| *name == s)
        .map(|&(_, t)| t)
        .ok_or_else(|| {
            let names: Vec<&str> = options.iter().map(|(n, _)| *n).collect();
            FileError::new(at, format!("must be one of {}", names.join(", ")))
        })
}

fn object<'a>(v: &'a Value, at: &str) -> Result<&'a Map<String, Value>, FileError> {
    v.as_object()
        .ok_or_else(|| FileError::new(at, "must be an object"))
}

fn known_keys(obj: &Map<String, Value>, at: &str, known: &[&str]) -> Result<(), FileError> {
    match obj.keys().find(|k| !known.contains(&k.as_str())) {
        Some(k) => Err(FileError::new(path(at, k), "is not a known field")),
        None => Ok(()),
    }
}

fn path(at: &str, key: &str) -> String {
    match at.is_empty() {
        true => key.to_owned(),
        false => format!("{at}.{key}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(text: &str) -> String {
        parse(text).unwrap_err().to_string()
    }

    #[test]
    fn reads_a_seat() {
        let d = parse(r#"{"v":1,"seats":[{"id":7,"x":6,"y":12.5,"facing":"W","depth":"deep","sides":["back"]}]}"#)
            .unwrap();
        let s = d.seats[0];
        assert_eq!(
            (s.id, s.at.y, s.facing, s.depth),
            (SeatId(7), Len::tenths(125), Dir::W, SeatDepth::Deep)
        );
        assert_eq!(
            s.sides,
            Sides {
                back: true,
                left: false,
                right: false
            }
        );
    }

    #[test]
    fn sides_are_optional() {
        let d = parse(r#"{"v":1,"seats":[{"id":1,"x":0,"y":0,"facing":"N","depth":"standard"}]}"#)
            .unwrap();
        assert_eq!(d.seats[0].sides, Sides::default());
    }

    #[test]
    fn refuses_bad_input_naming_the_field() {
        assert_eq!(err(r#"{"v":2,"seats":[]}"#), "v must be 1");
        assert_eq!(err(r#"{"v":1}"#), "seats must be a list");
        assert_eq!(
            err(r#"{"v":1,"seats":[],"extra":0}"#),
            "extra is not a known field"
        );
        let base = r#""id":1,"x":0,"y":0,"depth":"standard""#;
        assert_eq!(
            err(&format!(r#"{{"v":1,"seats":[{{{base}}}]}}"#)),
            "seats[0].facing is missing"
        );
        assert_eq!(
            err(&format!(r#"{{"v":1,"seats":[{{{base},"facing":"up"}}]}}"#)),
            "seats[0].facing must be one of N, E, S, W"
        );
        assert_eq!(
            err(&format!(
                r#"{{"v":1,"seats":[{{{base},"facing":"N","sides":["back","back"]}}]}}"#
            )),
            "seats[0].sides[1] repeats the back side"
        );
        assert_eq!(
            err(r#"{"v":1,"seats":[{"id":1,"x":0.25,"y":0,"facing":"N","depth":"deep"}]}"#),
            "seats[0].x must be in whole tenths of an inch"
        );
        assert_eq!(
            err(r#"{"v":1,"seats":[{"id":70000,"x":0,"y":0,"facing":"N","depth":"deep"}]}"#),
            "seats[0].id must be a whole number from 0 to 65535"
        );
        assert!(err("{").starts_with("not valid JSON"));
    }
}
