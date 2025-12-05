//! A cargo tool to appraise the quality of Rust dependencies.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[tokio::main]
#[cfg_attr(coverage_nightly, coverage(off))] // can't figure out how to get to 100% coverage of this function
async fn main() -> Result<(), ohno::AppError> {
    cargo_aprz_lib::run(std::env::args()).await
}
