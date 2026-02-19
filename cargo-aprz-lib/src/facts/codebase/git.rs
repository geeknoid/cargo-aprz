use super::provider::LOG_TARGET;
use crate::Result;
use chrono::{DateTime, Utc};
use core::time::Duration;
use ohno::{IntoAppError, bail};
use std::fs;
use std::path::Path;
use tokio::process::Command;
use url::Url;

const GIT_TIMEOUT: Duration = Duration::from_mins(5);

/// Convert a path to a UTF-8 string, returning an error if the path contains invalid UTF-8.
fn path_str(path: &Path) -> Result<&str> {
    path.to_str().into_app_err("invalid UTF-8 in repository path")
}

/// Result of a repository sync operation.
pub enum RepoStatus {
    /// Repository was successfully cloned or updated.
    Ok,
    /// Repository does not exist on the remote.
    NotFound,
}

/// Clone or update a git repository
pub async fn get_repo(repo_path: &Path, repo_url: &Url) -> Result<RepoStatus> {
    let start_time = std::time::Instant::now();

    let status = get_repo_core(repo_path, repo_url).await?;

    if matches!(status, RepoStatus::Ok) {
        log::debug!(target: LOG_TARGET, "Successfully prepared cached repository from '{repo_url}' in {:.3}s", start_time.elapsed().as_secs_f64());
    }

    Ok(status)
}

async fn get_repo_core(repo_path: &Path, repo_url: &Url) -> Result<RepoStatus> {
    let path_str = path_str(repo_path)?;

    if !repo_path.exists() {
        if let Some(parent) = repo_path.parent() {
            fs::create_dir_all(parent).into_app_err_with(|| format!("creating directory '{}'", parent.display()))?;
        }

        return clone_repo(path_str, repo_url).await;
    }

    // Verify it's a valid git repository before attempting update
    if !repo_path.join(".git").exists() {
        log::warn!(target: LOG_TARGET, "Cached repository path '{path_str}' exists but .git directory missing, re-cloning");
        fs::remove_dir_all(repo_path)
            .into_app_err_with(|| format!("removing potentially corrupt cached repository '{path_str}'"))?;
        return clone_repo(path_str, repo_url).await;
    }

    log::info!(target: LOG_TARGET, "Syncing repository '{repo_url}'");

    // First, try to fetch new commits
    // --filter=blob:none downloads only commit/tree objects, not file contents
    // --prune removes refs that no longer exist on remote
    // --force allows updating refs even if they're not fast-forward
    let output = run_git_with_timeout(&["-C", path_str, "fetch", "origin", "--filter=blob:none", "--prune", "--force"]).await?;

    if !output.status.success() {
        // Fetch failed - repository might be corrupted, try re-clone
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!(target: LOG_TARGET, "Git fetch failed ({}), removing and re-cloning", stderr.trim());
        fs::remove_dir_all(path_str).into_app_err_with(|| format!("removing stale cached repository '{path_str}'"))?;
        return clone_repo(path_str, repo_url).await;
    }

    // Reset to match remote HEAD (discard any local changes)
    let output = run_git_with_timeout(&["-C", path_str, "reset", "--hard", "origin/HEAD"]).await?;
    check_git_output(&output, "git reset")?;
    Ok(RepoStatus::Ok)
}

/// Check whether git stderr indicates the repository was not found on the remote.
fn is_repo_not_found(stderr: &str) -> bool {
    let stderr_lower = stderr.to_lowercase();
    stderr_lower.contains("not found") || stderr_lower.contains("does not exist")
}

async fn clone_repo(repo_path: &str, repo_url: &Url) -> Result<RepoStatus> {
    log::info!(target: LOG_TARGET, "Syncing repository '{repo_url}'");
    // --filter=blob:none creates a partial clone with full history but no blob contents
    let output = run_git_with_timeout(&[
        "clone",
        "--filter=blob:none",
        "--single-branch",
        "--no-tags",
        repo_url.as_str(),
        repo_path,
    ])
    .await?;

    if output.status.success() {
        return Ok(RepoStatus::Ok);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Clean up any partial clone directory left behind
    let path = Path::new(repo_path);
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }

    if is_repo_not_found(&stderr) {
        log::debug!(target: LOG_TARGET, "Repository '{repo_url}' not found on remote");
        return Ok(RepoStatus::NotFound);
    }

    bail!("git clone failed: {stderr}");
}

fn check_git_output(output: &std::process::Output, operation: &str) -> Result<()> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{operation} failed: {stderr}");
    }
    Ok(())
}

/// Count unique contributors in the repository
pub async fn count_contributors(repo_path: &Path) -> Result<u64> {
    let path_str = path_str(repo_path)?;
    // -s = summary (count only), -n = sort by count, -e = show emails
    // --all ensures we count contributors from all fetched refs, not just HEAD
    let output = run_git_with_timeout(&["-C", path_str, "shortlog", "-sne", "--all"]).await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git shortlog failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().count() as u64)
}

/// Commit statistics gathered from a single git log invocation.
pub struct CommitStats {
    /// Total number of commits.
    pub commit_count: u64,
    /// Timestamp of the first (oldest) commit.
    pub first_commit_at: DateTime<Utc>,
    /// Timestamp of the most recent commit.
    pub last_commit_at: DateTime<Utc>,
    /// Number of commits within each requested time window, in the same order as the input.
    pub commits_per_window: Vec<u64>,
}

/// Gather commit statistics from a single `git log` invocation.
///
/// Returns total count, first/last commit timestamps, and per-window commit counts
/// for each entry in `day_windows`. Uses Unix timestamps for efficient comparison.
pub async fn get_commit_stats(repo_path: &Path, day_windows: &[i64]) -> Result<CommitStats> {
    let path_str = path_str(repo_path)?;

    // %at = author date as Unix timestamp
    let output = run_git_with_timeout(&["-C", path_str, "log", "--format=%at"]).await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git log failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let now = Utc::now().timestamp();

    let mut commit_count: u64 = 0;
    let mut first_timestamp: Option<i64> = None;
    let mut last_timestamp: Option<i64> = None;
    let mut window_counts = vec![0u64; day_windows.len()];
    let window_thresholds: Vec<i64> = day_windows.iter().map(|days| now - days * 86400).collect();

    for line in stdout.lines() {
        let ts: i64 = match line.trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        commit_count += 1;

        // git log outputs newest first, so first parsed is last_timestamp, last parsed is first_timestamp
        if last_timestamp.is_none() {
            last_timestamp = Some(ts);
        }
        first_timestamp = Some(ts);

        for (i, threshold) in window_thresholds.iter().enumerate() {
            if ts >= *threshold {
                window_counts[i] += 1;
            }
        }
    }

    let first_commit_at = first_timestamp
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .unwrap_or(DateTime::UNIX_EPOCH);

    let last_commit_at = last_timestamp
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .unwrap_or(DateTime::UNIX_EPOCH);

    Ok(CommitStats {
        commit_count,
        first_commit_at,
        last_commit_at,
        commits_per_window: window_counts,
    })
}

async fn run_git_with_timeout(args: &[&str]) -> Result<std::process::Output> {
    let child = Command::new("git")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .into_app_err("spawning git command")?;

    match tokio::time::timeout(GIT_TIMEOUT, child.wait_with_output()).await {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => Err(e).into_app_err_with(|| format!("running 'git {}'", args.join(" "))),
        Err(_) => {
            bail!("'git {}' timed out after {} seconds", args.join(" "), GIT_TIMEOUT.as_secs());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{ExitStatus, Output};

    #[test]
    fn test_check_git_output_success() {
        #[cfg(unix)]
        let status = {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(0)
        };

        #[cfg(windows)]
        let status = {
            use std::os::windows::process::ExitStatusExt;
            ExitStatus::from_raw(0)
        };

        let output = Output {
            status,
            stdout: vec![],
            stderr: vec![],
        };

        check_git_output(&output, "test operation").unwrap();
    }

    #[test]
    fn test_check_git_output_failure() {
        #[cfg(unix)]
        let status = {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(256) // Exit code 1
        };

        #[cfg(windows)]
        let status = {
            use std::os::windows::process::ExitStatusExt;
            ExitStatus::from_raw(1)
        };

        let output = Output {
            status,
            stdout: vec![],
            stderr: b"error: failed to do something".to_vec(),
        };

        let result = check_git_output(&output, "test operation");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test operation failed"));
    }

    #[test]
    fn test_check_git_output_with_stderr() {
        #[cfg(unix)]
        let status = {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(256)
        };

        #[cfg(windows)]
        let status = {
            use std::os::windows::process::ExitStatusExt;
            ExitStatus::from_raw(1)
        };

        let stderr_msg = b"fatal: not a git repository";
        let output = Output {
            status,
            stdout: vec![],
            stderr: stderr_msg.to_vec(),
        };

        let result = check_git_output(&output, "git status");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("git status failed"));
        assert!(error_msg.contains("not a git repository"));
    }
}
