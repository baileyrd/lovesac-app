//! `planner check <layout.json>`: prints the parts list and size, or every problem.
//!
//! Exit codes: 0 the layout can be built, 1 it was refused, 2 usage or file error.

use std::process::ExitCode;

use planner_cli::{Outcome, check};

const USAGE: &str = "usage: planner check <layout.json>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [cmd, path] = args.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    if cmd != "check" {
        eprintln!("unknown command {cmd:?}\n{USAGE}");
        return ExitCode::from(2);
    }
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("can't read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    match check(&text) {
        Ok(Outcome::Valid(report)) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Ok(Outcome::Refused(report)) => {
            print!("{report}");
            ExitCode::from(1)
        }
        Err(e) => {
            eprintln!("{path}: {e}");
            ExitCode::from(2)
        }
    }
}
