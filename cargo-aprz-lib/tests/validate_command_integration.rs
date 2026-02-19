//! Integration test for the `validate` command.
//!
//! Exercises the path where no explicit `--config` is provided, causing
//! `validate_config` to resolve the workspace root via `MetadataCommand`
//! (line 59 of validate.rs).
//!
//! This test does NOT require the `network_tests` feature because it only
//! exercises local config validation logic (no network access).

use cargo_aprz_lib::Host;
use std::io::Cursor;

/// Test host that captures output to in-memory buffers.
struct TestHost {
    output_buf: Vec<u8>,
    error_buf: Vec<u8>,
    exit_code: Option<i32>,
}

impl TestHost {
    const fn new() -> Self {
        Self {
            output_buf: Vec::new(),
            error_buf: Vec::new(),
            exit_code: None,
        }
    }

    fn error_str(&self) -> String {
        String::from_utf8_lossy(&self.error_buf).into_owned()
    }
}

impl Host for TestHost {
    fn output(&mut self) -> impl std::io::Write {
        Cursor::new(&mut self.output_buf)
    }

    fn error(&mut self) -> impl std::io::Write {
        Cursor::new(&mut self.error_buf)
    }

    fn exit(&mut self, code: i32) {
        self.exit_code = Some(code);
    }
}

/// Validate without an explicit config path when no `aprz.toml` exists
/// should produce an error.
#[tokio::test]
#[cfg_attr(miri, ignore = "Miri cannot call CreateIoCompletionPort")]
async fn test_validate_without_config_file_errors() {
    let mut host = TestHost::new();
    cargo_aprz_lib::run(
        &mut host,
        [
            "cargo",
            "aprz",
            "validate",
            "--manifest-path",
            "tests/fixtures/tiny-crate/Cargo.toml",
        ],
    )
    .await;

    assert_eq!(host.exit_code, Some(1), "validate should fail when no aprz.toml exists");
    assert!(
        host.error_str().contains("could not find configuration file"),
        "should report missing config file, got: {}",
        host.error_str()
    );
}
