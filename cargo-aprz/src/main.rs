//! A cargo tool to appraise the quality of Rust dependencies.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use cargo_aprz_lib::{Host, run};
use std::io::{Write, stderr, stdout};

/// Default host that runs real OS commands.
#[derive(Debug, Clone, Default)]
pub struct RealHost;

#[cfg_attr(coverage_nightly, coverage(off))]
impl Host for RealHost {
    fn output(&mut self) -> impl Write {
        stdout()
    }

    fn error(&mut self) -> impl Write {
        stderr()
    }

    fn exit(&mut self, code: i32) {
        std::process::exit(code);
    }
}

#[tokio::main]
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(test, mutants::skip)] // thin wrapper, tested via integration tests on `run()`
async fn main() {
    run(&mut RealHost, std::env::args()).await;
}
