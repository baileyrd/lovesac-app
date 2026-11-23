# planner (placeholder name)

A small planner for modular sofas built from rectangular seats and sides. `planner` is a
**placeholder**: the real name is still to pick, and it stays neutral (no maker's
trademarks in it).

This is the first slice from the spec: a pure Rust core plus a command-line check. No room,
no UI yet.

```
crates/core   planner-core  std only, no I/O: pieces, layout rules, parts list, footprint
crates/cli    planner-cli   `planner` binary: JSON layout file in, text report out
examples/     layout files used by the tests
```

## Run it (PowerShell)

```powershell
cargo run -p planner-cli -- check examples\nano-order.json
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Exit codes: `0` the layout can be built, `1` refused, `2` usage or file error.

```
Layout OK: 7 seats, 9 sides
Seats: 4 deep, 3 standard
Sides: 7 standard, 2 deep
Size:  128 x 117 in (10 ft 8 in x 9 ft 9 in)
```

## Layout file (v1)

Inches (tenths allowed). `x`/`y` is the seat's north-west corner, `y` grows south.
`facing` is the way the sitter faces; `sides` lists the edges carrying a side. Side
kinds aren't written: the core works them out.

```json
{ "v": 1,
  "seats": [
    { "id": 1, "x": 6, "y": 6, "facing": "S", "depth": "deep", "sides": ["back", "right"] }
  ] }
```

Unknown keys, repeated sides and off-grid numbers are refused with the field's path.

## Rules

Refused (every problem reported at once): no seats, duplicate ids, overlapping pieces, a
seat not joined edge to whole edge to the rest, and a side that no piece fits.

**Side fit is derived and unverified.** A side covers its seat edge plus 6 in for each
back corner it owns; where a back and an arm meet, the side on the 29 in edge owns the
corner. 35 in takes a standard side, 29 in a deep side (backs only), anything else is
refused. It reproduces the published 4 seats + 5 sides size and the order above, but it
needs checking against a real set or the assembly guide.

## Dependencies

- `planner-core`: none.
- `planner-cli`: `serde_json` only (untyped `Value`, no derive or proc macros). The spec
  fixes JSON as the file format for share links and the web binding, and a hand-rolled
  JSON parser would be more code to trust than one ubiquitous crate.
