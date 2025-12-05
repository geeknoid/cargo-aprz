# cargo-aprz

[![crate.io](https://img.shields.io/crates/v/cargo-aprz.svg)](https://crates.io/crates/cargo-aprz)
[![CI](https://github.com/geeknoid/cargo-aprz/workflows/main/badge.svg)](https://github.com/geeknoid/cargo-aprz/actions)
[![Coverage](https://codecov.io/gh/geeknoid/cargo-aprz/graph/badge.svg?token=FCUG0EL5TI)](https://codecov.io/gh/geeknoid/cargo-aprz)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Table of Contents

- [Summary](#summary)
- [Background](#background)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Collected Metrics](#collected-metrics)
  - [Data Sources](#data-sources)
  - [Metadata Metrics](#metadata-metrics)
  - [Documentation Metrics](#documentation-metrics)
  - [Usage Metrics](#usage-metrics)
  - [Stability Metrics](#stability-metrics)
  - [Ownership Metrics](#ownership-metrics)
  - [Community Metrics](#community-metrics)
  - [Activity Metrics](#activity-metrics)
  - [Advisory Metrics](#advisory-metrics)
  - [Code Metrics](#code-metrics)
  - [Trustworthiness Metrics](#trustworthiness-metrics)
  - [Report Formats](#report-formats)
  - [Display Options](#display-options)
  - [Logging](#logging)
- [Configuration](#configuration)
  - [Configuration Structure](#configuration-structure)
  - [Evaluation Logic](#evaluation-logic)
  - [Expression Syntax](#expression-syntax)
  - [Special Variables](#special-variables)
- [Expression Examples](#expression-examples)
  - [Security-Focused Policy](#security-focused-policy)
  - [Popularity-Based Acceptance](#popularity-based-acceptance)
  - [Quality Gate](#quality-gate)
  - [Complex Conditional Logic](#complex-conditional-logic)
- [Complete Workflow Examples](#complete-workflow-examples)
  - [Example 1: Initial Setup](#example-1-initial-setup)
  - [Example 2: Security Audit](#example-2-security-audit)
  - [Example 3: Generate Reports](#example-3-generate-reports)
  - [Example 4: Compare Alternatives](#example-4-compare-alternatives)
  - [Example 5: CI Integration](#example-5-ci-integration)
  - [Example 6: Metrics-Only Analysis](#example-6-metrics-only-analysis)
- [Troubleshooting](#troubleshooting)
  - [Expression Validation Errors](#expression-validation-errors)
  - [GitHub Rate Limiting](#github-rate-limiting)
  - [Missing Metric Data](#missing-metric-data)
  - [Crate Not Found](#crate-not-found)
  - [Performance](#performance)

## Summary

A cargo tool to appraise the quality of Rust dependencies.

## Background

Building modern applications often involves integrating a large number of third-party dependencies.
While these dependencies can provide valuable functionality and accelerate development, they also
introduce risks related to quality, security vulnerabilities, future compatibility.

Before taking a dependency in your project, it's useful to vet whether that dependency meets some baseline
quality standards. For example, maybe you believe in having excellent unit test coverage for your projects,
but if you pull in some dependency which has no tests, it can undermine the quality of your application.

`cargo-aprz` leta you approise the quality of dependencies. For any given crate, it collects a large number
of metrics, such as the number of open issues, the frequency of releases, the presence of a security advisory,
the number of examples, the code coverage percentage, and many more. You can nice reports showing you
all of these metrics in an easy to consume form.

You can also use `cargo-aprz` to automatically evaluate whether a crate meets your quality standards. You
do this by writing a set of expressions that operate on the collected metrics. For example, you can have an
expression that says "if code coverage is less than 20%, treat this crate as not being acceptable as a dependency".

You can run `cargo-aprz` by specifying a set of crates to evaluate, or you can run it on all the full transitive set of
dependencies for an existing project.

## Installation

```bash
cargo install --locked cargo-aprz
```

## Quick Start

1. Generate a default configuration file:

   ```bash
   cargo aprz init
   ```

   This creates `aprz.toml` which will let you control various options. This file also contains a full list of the
   metrics this tool collects, along with basic descriptions of these metrics.

2. Get the metrics associated with a crate:

   ```bash
   cargo aprz crates tokio
   ```

   The first time you run this command, it will take a while as it needs to download
   a large database from crates.io along with the `RustSec` advisory database. This
   data is cached such that subsequent runs will be much faster.

3. Get the metrics associated with the dependencies of a Rust project:

   ```bash
   cargo aprz deps
   ```

4. Get the metrics for specific versions of crates:

   ```bash
   cargo aprz crates tokio@1.40.0 serde@1.0.0
   ```

5. Get the metrics for a crate and produce an HTML report instead of outputting to the console:

   ```bash
   cargo aprz crates tokio@1.40.0 --html report.html
   ```

## Collected Metrics

`cargo-aprz` collects a wide variety of metrics from a variety of different sources. Each metric has
a name and a category, helping you understand what the metric is describing.

### Data Sources

`cargo-aprz` collects data from a variety of sources:

- **crates.io**: Provides metadata and download statistics for each crate.

- **GitHub** or **Codeberg**: Provides information about the popularity of a crate, the
  number of issues and pull requests, the frequency of commits, and more. This is also
  where `cargo-aprz` gets source code in order to analyze the code quality of a crate.

- **`RustSec` Advisory Database**: Provides information about known security vulnerabilities in Rust crates.

- **docs.rs**: Provides information about the quality of documentation for a crate, such as the presence of examples,
  the number of items with documentation comments, and more.

- **codecov.io**: Provides code coverage information.

### Metadata Metrics

| Metric | Description |
|--------|-------------|
| `crate.name` | Name of the crate |
| `crate.version` | Semantic version of the crate |
| `crate.description` | Description of the crate's purpose and use |
| `crate.license` | SPDX license identifier constraining use of the crate |
| `crate.categories` | Crate categories |
| `crate.keywords` | Crate keywords |
| `crate.features` | Available crate features |
| `crate.repository` | URL to the crate's source code repository |
| `crate.homepage` | URL to the crate's homepage |
| `crate.minimum_rust` | Minimum Rust version (MSRV) required to compile this crate |
| `crate.rust_edition` | Rust edition this crate targets |

### Documentation Metrics

| Metric | Description |
|--------|-------------|
| `docs.documentation` | URL to the crate's documentation |
| `docs.public_api_elements` | Number of public API elements (functions, structs, etc.) |
| `docs.undocumented_public_api_elements` | Number of public API elements without documentation |
| `docs.public_api_coverage_percentage` | Percentage of public API elements with documentation |
| `docs.crate_level_docs_present` | Whether crate-level documentation exists |
| `docs.broken_links` | Number of broken links in documentation |
| `docs.examples_in_docs` | Number of code examples in documentation |
| `docs.standalone_examples` | Number of standalone example programs in the codebase |

### Usage Metrics

| Metric | Description |
|--------|-------------|
| `usage.total_downloads` | Crate downloads across all versions |
| `usage.total_downloads_last_90_days` | Crate downloads across all versions in the last 90 days |
| `usage.version_downloads` | Crate downloads of this specific version |
| `usage.version_downloads_last_90_days` | Crate downloads of this specific version in the last 90 days |
| `usage.dependent_crates` | Number of unique crates that depend on this crate |

### Stability Metrics

| Metric | Description |
|--------|-------------|
| `stability.crate_created_at` | When the crate was first published to crates.io |
| `stability.crate_updated_at` | When the crate's metadata was last updated on crates.io |
| `stability.version_created_at` | When this version was first published to crates.io |
| `stability.version_updated_at` | When this version's metadata was last updated on crates.io |
| `stability.yanked` | Whether this version has been yanked from crates.io |
| `stability.versions_last_90_days` | Number of versions published in the last 90 days |

### Ownership Metrics

| Metric | Description |
|--------|-------------|
| `owners.names` | List of owner usernames |

### Community Metrics

| Metric | Description |
|--------|-------------|
| `community.repo_stars` | Number of stars on the repository |
| `community.repo_forks` | Number of forks of the repository |
| `community.repo_subscribers` | Number of users watching/subscribing to the repository |
| `community.repo_contributors` | Number of contributors to the repository |

### Activity Metrics

| Metric | Description |
|--------|-------------|
| `activity::commits_last_90_days` | Number of commits to the repository in the last 90 days |
| `activity.open_issues` | Number of currently open issues |
| `activity.closed_issues` | Total number of issues that have been closed (all time) |
| `activity.avg_open_issue_age_days` | Average age in days of open issues |
| `activity.median_open_issue_age_days` | Median age in days of open issues (50th percentile) |
| `activity.p90_open_issue_age_days` | 90th percentile age in days of open issues |
| `activity.open_pull_requests` | Number of currently open pull requests |
| `activity.closed_pull_requests` | Total number of pull requests that have been closed (all time) |
| `activity.avg_open_pull_request_age_days` | Average age in days of open pull requests |
| `activity.median_open_pull_request_age_days` | Median age in days of open pull requests (50th percentile) |
| `activity.p90_open_pull_request_age_days` | 90th percentile age in days of open pull requests |

#### Advisory Metrics

| Metric | Description |
|--------|-------------|
| `advisories.total_low_severity_vulnerabilities` | Number of low severity vulnerabilities across all versions |
| `advisories.total_medium_severity_vulnerabilities` | Number of medium severity vulnerabilities across all versions |
| `advisories.total_high_severity_vulnerabilities` | Number of high severity vulnerabilities across all versions |
| `advisories.total_critical_severity_vulnerabilities` | Number of critical severity vulnerabilities across all versions |
| `advisories.total_notice_warnings` | Number of notice warnings across all versions |
| `advisories.total_unmaintained_warnings` | Number of unmaintained warnings across all versions |
| `advisories.total_unsound_warnings` | Number of unsound warnings across all versions |
| `advisories.version_low_severity_vulnerabilities` | Number of low severity vulnerabilities in this version |
| `advisories.version_medium_severity_vulnerabilities` | Number of medium severity vulnerabilities in this version |
| `advisories.version_high_severity_vulnerabilities` | Number of high severity vulnerabilities in this version |
| `advisories.version_critical_severity_vulnerabilities` | Number of critical severity vulnerabilities in this version |
| `advisories.version_notice_warnings` | Number of notice warnings for this version |
| `advisories.version_unmaintained_warnings` | Number of unmaintained warnings for this version |
| `advisories.version_unsound_warnings` | Number of unsound warnings for this version |

#### Code Metrics

| Metric | Description |
|--------|-------------|
| `code.source_files` | Number of source files |
| `code.source_files_with_errors` | Number of source files that had analysis errors |
| `code.code_lines` | Number of lines of production code (excluding tests) |
| `code.test_lines` | Number of lines of test code |
| `code.comment_lines` | Number of comment lines in the codebase |
| `code.transitive_dependencies` | Number of transitive dependencies |

#### Trustworthiness Metrics

| Metric | Description |
|--------|-------------|
| `trust.unsafe_blocks` | Number of unsafe blocks in the codebase |
| `trust.ci_workflows` | Whether CI/CD workflows were detected in the repository |
| `trust.miri_usage` | Whether Miri is used in CI |
| `trust.clippy_usage` | Whether Clippy is used in CI |
| `trust.code_coverage_percentage` | Percentage of code covered by tests |

### Report Formats

`cargo-aprz` can output reports in multiple formats. By default, it outputs a human-readable table to the console. You can
also output in JSON format, HTML format, CSV, or Excel.

```bash
cargo aprz deps                     # Terminal output (default)
cargo aprz deps --html report.html  # HTML report
cargo aprz deps --excel report.xlsx # Excel spreadsheet
cargo aprz deps --csv report.csv    # CSV file
cargo aprz deps --json report.json  # JSON data
```

## EVERYTHING BELOW IS AI GENERATED. SORRY, I'LL FIX IT SOON TO BE SANE.

### Display Options

```bash
cargo aprz deps --show-ranking     # Show ACCEPTED/DENIED status
cargo aprz deps --color always     # Force colored output
cargo aprz deps --color never      # Disable colored output
```

### Logging

```bash
cargo aprz deps --log-level info   # info, warn, error, debug
cargo aprz deps --log-level debug  # Verbose debugging output
```

## Configuration

Configuration files define policy expressions that determine whether crates should
be accepted or denied. The configuration uses TOML format with three types of
expression lists.

### Configuration Structure

```toml
# Deny if ANY of these expressions evaluate to true
[[deny_if_any]]
name = "critical_vulnerabilities"
description = "Deny crates with critical security vulnerabilities"
expression = "advisories.total_critical_severity_vulnerabilities > 0"

[[deny_if_any]]
name = "too_many_dependencies"
expression = "code.transitive_dependencies > 200"

# Accept if ANY of these expressions evaluate to true (unless denied)
[[accept_if_any]]
name = "very_popular"
description = "Auto-accept extremely popular crates"
expression = "community.repo_stars > 5000"

# Accept ONLY if ALL of these expressions evaluate to true (unless denied)
[[accept_if_all]]
name = "good_coverage"
description = "Must have good test coverage"
expression = "trust.code_coverage_percentage >= 70.0"

[[accept_if_all]]
name = "no_current_vulnerabilities"
expression = "advisories.total_vulnerabilities == 0"
```

### Evaluation Logic

When `--eval` is enabled, crates are evaluated in this order:

1. **Deny-if-any**: If ANY expression evaluates to true → DENIED
2. **Accept-if-any**: If ANY expression evaluates to true → ACCEPTED
3. **Accept-if-all**: If ALL expressions evaluate to true → ACCEPTED
4. Otherwise → NOT EVALUATED

This allows flexible policies like:
- "Deny any crate with critical vulnerabilities"
- "Auto-accept crates with >10k stars"
- "Accept only if coverage ≥80% AND no vulnerabilities"

### Expression Syntax

Expressions use CEL (Common Expression Language) syntax and can reference
any collected metric by name. Expressions must evaluate to a boolean.

**Comparison operators:**
```text
==  !=  <  <=  >  >=
```

**Logical operators:**
```text
&&  ||  !
```

**Ternary operator:**
```text
condition ? true_value : false_value
```

**Null handling:**
```text
metric == null
metric != null ? metric > 100 : false
```


#### Special Variables
```text
now - Current timestamp (datetime), useful for date comparisons
```

## Expression Examples

### Security-Focused Policy

Deny any crate with security issues:

```toml
[[deny_if_any]]
name = "critical_vulns"
expression = "advisories.total_critical_severity_vulnerabilities > 0"

[[deny_if_any]]
name = "high_vulns"
expression = "advisories.total_high_severity_vulnerabilities > 0"

[[deny_if_any]]
name = "unmaintained"
expression = "advisories.total_unmaintained_warnings > 0"
```

### Popularity-Based Acceptance

Auto-accept widely-used crates:

```toml
[[accept_if_any]]
name = "very_popular"
description = "Highly starred projects are generally trustworthy"
expression = "community.repo_stars > 5000"

[[accept_if_any]]
name = "high_usage"
description = "Many downloads indicate community trust"
expression = "usage.total_downloads > 10000000"
```

### Quality Gate

Accept only crates meeting all quality criteria:

```toml
[[accept_if_all]]
name = "good_coverage"
expression = "trust.code_coverage_percentage >= 80.0"

[[accept_if_all]]
name = "good_docs"
expression = "docs.public_api_coverage_percentage >= 90.0"

[[accept_if_all]]
name = "active_maintenance"
expression = "activity.commits_last_90_days >= 5"

[[accept_if_all]]
name = "no_vulnerabilities"
expression = "advisories.total_vulnerabilities == 0"
```

### Complex Conditional Logic

Use ternary operators for nuanced policies:

```toml
[[accept_if_any]]
name = "popular_or_well_tested"
description = "Either popular OR has excellent coverage"
expression = "community.repo_stars > 1000 || trust.code_coverage_percentage >= 95.0"

[[accept_if_all]]
name = "reasonable_dependencies"
description = "Dependency count varies by crate popularity"
expression = "usage.total_downloads > 1000000 ? code.transitive_dependencies < 100 : code.transitive_dependencies < 50"
```

## Complete Workflow Examples

### Example 1: Initial Setup

Set up cargo-aprz for your project:

```bash
# Create default configuration
cargo aprz init

# Edit aprz.toml to customize policies
# ...

# Validate configuration
cargo aprz validate

# Analyze dependencies
cargo aprz deps --eval --show-ranking
```

### Example 2: Security Audit

Check for security vulnerabilities:

```bash
# Create security-focused config
cat > security.toml << 'EOF'
[[deny_if_any]]
name = "any_vulnerabilities"
expression = "advisories.total_vulnerabilities > 0"

[[deny_if_any]]
name = "unmaintained"
expression = "advisories.total_unmaintained_warnings > 0"
EOF

# Run security check
cargo aprz deps --config security.toml --eval --show-ranking
```

### Example 3: Generate Reports

Create comprehensive reports in multiple formats:

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
cargo aprz deps \
  --eval \
  --show-ranking \
  --html report.html \
  --excel report.xlsx \
  --json report.json
```

### Example 4: Compare Alternatives

Evaluate competing crates before choosing:

```bash
cargo aprz crates tokio async-std smol --console
```

### Example 5: CI Integration

GitHub Actions workflow:

```yaml
name: Dependency Policy Check

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-aprz
        run: cargo install cargo-aprz

      - name: Validate config
        run: cargo aprz validate

      - name: Check dependencies
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: cargo aprz deps --eval --show-ranking --html report.html

      - name: Upload report
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: dependency-report
          path: report.html
```

### Example 6: Metrics-Only Analysis

Collect and report metrics without policy evaluation:

```bash
cargo aprz deps --excel metrics.xlsx  # No --eval flag
```

## Troubleshooting

### Expression Validation Errors

If expressions fail to parse:
```bash
cargo aprz validate  # Check configuration syntax
```

Common issues:
- Typo in metric names (check spelling against available metrics)
- Expression doesn't return boolean
- Invalid CEL syntax (missing quotes, unbalanced parentheses)

### GitHub Rate Limiting

Without authentication: 60 requests/hour
With token: 5000 requests/hour

Set `GITHUB_TOKEN` environment variable to increase limits.

### Missing Metric Data

Some metrics may be null if data sources are unavailable:
- Use null checks in expressions: `metric != null && metric > 100`
- Use ternary: `metric != null ? metric > 100 : false`

### Crate Not Found

If a crate isn't found on crates.io:
- Verify crate name spelling
- Check if crate has been yanked
- Ensure network connectivity
