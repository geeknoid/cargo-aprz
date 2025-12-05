//! Rule evaluation logic for crates.

use crate::config::{Config, Policy, ResponsivenessPolicy};
use crate::facts::AgeStats;
use crate::facts::OwnerKind;
use crate::facts::{CrateFacts, ProviderResult};
use crate::metrics::Metric;
use crate::misc::DependencyType;
use crate::ranking::PolicyOutcome;
use chrono::{Duration, Utc};
use std::collections::HashMap;

struct MetricCalculator<'a> {
    config: &'a Config,
    facts: &'a CrateFacts,
    dependency_type: DependencyType,
    results: &'a mut HashMap<Metric, PolicyOutcome>,
}

/// Calculate all the metrics for a given crate
pub fn calculate(config: &Config, facts: &CrateFacts, dependency_type: DependencyType, results: &mut HashMap<Metric, PolicyOutcome>) {
    let mut calc = MetricCalculator {
        config,
        facts,
        dependency_type,
        results,
    };

    calc.license();
    calc.age();
    calc.min_version();
    calc.release_count();
    calc.overall_download_count();
    calc.one_month_download_count();
    calc.overall_owner_count();
    calc.team_owner_count();
    calc.user_owner_count();
    calc.direct_dependency_count();
    calc.dependent_count();

    calc.doc_coverage_percentage();
    calc.broken_doc_link_count();
    calc.code_coverage_percentage();
    calc.fully_safe_code();
    calc.transitive_dependency_count();
    calc.example_count();

    calc.repo_contributor_count();
    calc.repo_star_count();
    calc.repo_fork_count();
    calc.repo_subscriber_count();
    calc.commit_activity();
    calc.open_issue_count();
    calc.closed_issue_count();
    calc.issue_responsiveness();
    calc.open_pull_request_count();
    calc.closed_pull_request_count();
    calc.pull_request_responsiveness();

    calc.vulnerability_count();
    calc.low_vulnerability_count();
    calc.medium_vulnerability_count();
    calc.high_vulnerability_count();
    calc.critical_vulnerability_count();
    calc.warning_count();
    calc.notice_warning_count();
    calc.unmaintained_warning_count();
    calc.unsound_warning_count();
    calc.yanked_warning_count();

    calc.historical_vulnerability_count();
    calc.historical_low_vulnerability_count();
    calc.historical_medium_vulnerability_count();
    calc.historical_high_vulnerability_count();
    calc.historical_critical_vulnerability_count();
    calc.historical_warning_count();
    calc.historical_notice_warning_count();
    calc.historical_unmaintained_warning_count();
    calc.historical_unsound_warning_count();
    calc.historical_yanked_warning_count();
}

impl MetricCalculator<'_> {
    /// Evaluate if the crate's license is acceptable.
    fn license(&mut self) {
        let ProviderResult::Found(crate_version_data) = &self.facts.crate_version_data else {
            unreachable!("analyzable crate must have Found data");
        };
        let license = &crate_version_data.license;
        let license_str = license.as_deref().unwrap_or("None");

        self.apply_generic_policy(
            Metric::License,
            &self.config.license,
            |p| license.as_ref().is_some_and(|l| p.check_license(l)),
            |_| format!("'{license_str}'"),
            || format!("'{license_str}'; not a supported license type"),
        );
    }

    /// Evaluate the age of the crate (time since first version was released).
    fn age(&mut self) {
        let now = Utc::now();
        let ProviderResult::Found(crate_overall_data) = &self.facts.crate_overall_data else {
            unreachable!("analyzable crate must have Found data");
        };
        let crate_creation_date = crate_overall_data.created_at;
        let age_days = u64::try_from((now - crate_creation_date).num_days().max(0)).unwrap_or(0);

        let min_days = &self
            .config
            .age
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_days))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::Age,
            &self.config.age,
            |p| age_days >= u64::from(p.min_days),
            |_| format!("{age_days} days"),
            || format!("{age_days} days (need >= {min_days})"),
        );
    }

    /// Evaluate if the crate has reached a stable version (1.0+).
    fn min_version(&mut self) {
        let ProviderResult::Found(crate_version_data) = &self.facts.crate_version_data else {
            unreachable!("analyzable crate must have Found data");
        };
        let major_version = crate_version_data.version.major;

        let min_version = &self
            .config
            .min_version
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_major_version))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::MinVersion,
            &self.config.min_version,
            |p| major_version >= u64::from(p.min_major_version),
            |_| format!("v{major_version}"),
            || format!("v{major_version} (need >= v{min_version})"),
        );
    }

    /// Evaluate how frequently the crate is released.
    fn release_count(&mut self) {
        self.apply_generic_policy(
            Metric::ReleaseCount,
            &self.config.release_count,
            |p| count_releases_in_period(self.facts, p.max_days) >= p.min_count as usize,
            |p| {
                format!(
                    "{} releases in {} days",
                    count_releases_in_period(self.facts, p.max_days),
                    p.max_days,
                )
            },
            || "insufficient recent releases".to_string(),
        );
    }

    /// Evaluate overall download count since publication.
    fn overall_download_count(&mut self) {
        let ProviderResult::Found(crate_overall_data) = &self.facts.crate_overall_data else {
            unreachable!("analyzable crate must have Found data");
        };
        let downloads = crate_overall_data.downloads;

        let min_downloads = &self
            .config
            .overall_download_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::OverallDownloadCount,
            &self.config.overall_download_count,
            |p| downloads >= u64::from(p.min_count),
            |_| format!("{downloads} total downloads"),
            || format!("{downloads} total downloads (need >= {min_downloads})"),
        );
    }

    /// Evaluate download count in the last month.
    fn one_month_download_count(&mut self) {
        let ProviderResult::Found(crate_overall_data) = &self.facts.crate_overall_data else {
            unreachable!("analyzable crate must have Found data");
        };
        // Get the most recent month's downloads (last entry in the vector)
        let recent_downloads = crate_overall_data.monthly_downloads.last().map_or(0, |(_, downloads)| *downloads);

        let min_downloads = &self
            .config
            .one_month_download_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::OneMonthDownloadCount,
            &self.config.one_month_download_count,
            |p| recent_downloads >= u64::from(p.min_count),
            |_| format!("{recent_downloads} downloads in the last month"),
            || format!("{recent_downloads} downloads in the last month (need >= {min_downloads})"),
        );
    }

    /// Evaluate the total number of owners (users + teams).
    fn overall_owner_count(&mut self) {
        let ProviderResult::Found(crate_overall_data) = &self.facts.crate_overall_data else {
            unreachable!("analyzable crate must have Found data");
        };
        let owner_count = u64::try_from(crate_overall_data.owners.len()).expect("owner count always fits in u64");

        let min_owner_count = &self
            .config
            .overall_owner_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::OverallOwnerCount,
            &self.config.overall_owner_count,
            |p| owner_count >= u64::from(p.min_count),
            |_| format!("{owner_count} total owners"),
            || format!("{owner_count} total owners (need >= {min_owner_count})"),
        );
    }

    /// Evaluate the number of team owners.
    fn team_owner_count(&mut self) {
        let owner_team_count = self.get_owner_count(OwnerKind::Team);

        let min_owner_team_count = &self
            .config
            .team_owner_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::TeamOwnerCount,
            &self.config.team_owner_count,
            |p| owner_team_count >= u64::from(p.min_count),
            |_| format!("{owner_team_count} team owners"),
            || format!("{owner_team_count} team owners (need >= {min_owner_team_count})"),
        );
    }

    /// Evaluate the number of user owners.
    fn user_owner_count(&mut self) {
        let owner_user_count = self.get_owner_count(OwnerKind::User);

        let min_owner_user_count = &self
            .config
            .user_owner_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::UserOwnerCount,
            &self.config.user_owner_count,
            |p| owner_user_count >= u64::from(p.min_count),
            |_| format!("{owner_user_count} user owners"),
            || format!("{owner_user_count} user owners (need >= {min_owner_user_count})"),
        );
    }

    /// Evaluate the number of direct dependencies (fewer is better).
    #[expect(
        clippy::unused_self,
        clippy::missing_const_for_fn,
        reason = "Disabled placeholder until direct_dependencies available from CodebaseData"
    )]
    fn direct_dependency_count(&self) {
        // Note: Direct dependency count is not currently available from crates.io data.
        // It will need to be sourced from CodebaseData (via cargo metadata).

        /* Disabled until direct_dependencies is available from source_data
        let direct_deps = self.facts.crate_version_data.direct_dependencies;

        let max_direct_deps = &self
            .config
            .direct_dependency_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.max_count))
            .max()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::DirectDependencyCount,
            &self.config.direct_dependency_count,
            |p| direct_deps <= u64::from(p.max_count),
            |_| format!("{direct_deps} direct dependencies"),
            || format!("{direct_deps} direct dependencies (need < {max_direct_deps})"),
        );
        */
    }

    /// Evaluate the number of dependents (more is better).
    fn dependent_count(&mut self) {
        let ProviderResult::Found(crate_overall_data) = &self.facts.crate_overall_data else {
            unreachable!("analyzable crate must have Found data");
        };
        let deps = crate_overall_data.dependents;

        let min_deps = &self
            .config
            .dependent_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::DependentCount,
            &self.config.dependent_count,
            |p| deps >= u64::from(p.min_count),
            |_| format!("{deps} dependents"),
            || format!("{deps} dependents (need >= {min_deps})"),
        );
    }

    /// Evaluate documentation coverage percentage.
    fn doc_coverage_percentage(&mut self) {
        let ProviderResult::Found(docs_data) = &self.facts.docs_data else {
            return;
        };

        // Return early if format version is unknown
        let crate::facts::MetricState::Found(metrics) = &docs_data.state else {
            return;
        };

        let doc_coverage = metrics.doc_coverage_percentage;

        let min_coverage = &self
            .config
            .doc_coverage_percentage
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_percentage))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::DocCoveragePercentage,
            &self.config.doc_coverage_percentage,
            |p| doc_coverage >= p.min_percentage,
            |_| format!("{doc_coverage}% documentation coverage"),
            || format!("{doc_coverage}% documentation coverage (need >= {min_coverage}%)"),
        );
    }

    /// Evaluate the number of broken documentation links.
    fn broken_doc_link_count(&mut self) {
        let ProviderResult::Found(docs_data) = &self.facts.docs_data else {
            return;
        };

        // Return early if format version is unknown
        let crate::facts::MetricState::Found(metrics) = &docs_data.state else {
            return;
        };

        let broken_links = metrics.broken_doc_links;

        let max_broken_links = &self
            .config
            .broken_doc_link_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.max_count))
            .max()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::BrokenDocLinkCount,
            &self.config.broken_doc_link_count,
            |p| broken_links <= u64::from(p.max_count),
            |_| format!("{broken_links} broken documentation links"),
            || format!("{broken_links} broken documentation links (need < {max_broken_links})"),
        );
    }

    /// Evaluate codebase coverage percentage.
    fn code_coverage_percentage(&mut self) {
        let ProviderResult::Found(coverage_data) = &self.facts.coverage_data else {
            return;
        };

        let code_coverage = coverage_data.code_coverage_percentage;

        let min_coverage = &self
            .config
            .code_coverage_percentage
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| f64::from(p.min_percentage))
            .min_by(|a, b| a.partial_cmp(b).expect("percentage values should not be NaN"))
            .unwrap_or(0.0);

        self.apply_generic_policy(
            Metric::CodeCoveragePercentage,
            &self.config.code_coverage_percentage,
            |p| code_coverage >= f64::from(p.min_percentage),
            |_| format!("{code_coverage:.1}% codebase coverage"),
            || format!("{code_coverage:.1}% codebase coverage (need >= {min_coverage:.1}%)"),
        );
    }

    /// Evaluate for unsafe codebase presence.
    fn fully_safe_code(&mut self) {
        let ProviderResult::Found(source_data) = &self.facts.codebase_data else {
            return;
        };
        let has_unsafe = source_data.unsafe_count > 0;

        self.apply_generic_policy(
            Metric::FullySafeCode,
            &self.config.fully_safe_code,
            |_| !has_unsafe,
            |_| "crate contains no unsafe codebase".to_string(),
            || "crate contains unsafe codebase".to_string(),
        );
    }

    /// Evaluate the number of transitive dependencies (fewer is better).
    fn transitive_dependency_count(&mut self) {
        let ProviderResult::Found(source_data) = &self.facts.codebase_data else {
            return;
        };
        let transitive_deps = source_data.transitive_dependencies;

        let max_deps = &self
            .config
            .transitive_dependency_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.max_count))
            .max()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::TransitiveDependencyCount,
            &self.config.transitive_dependency_count,
            |p| transitive_deps <= u64::from(p.max_count),
            |_| format!("{transitive_deps} transitive dependencies"),
            || format!("{transitive_deps} transitive dependencies (need < {max_deps})"),
        );
    }

    /// Evaluate the number of codebase examples (more is better).
    fn example_count(&mut self) {
        let ProviderResult::Found(source_data) = &self.facts.codebase_data else {
            return;
        };
        let example_count = source_data.example_count;

        let min_examples = &self
            .config
            .example_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::ExampleCount,
            &self.config.example_count,
            |p| example_count >= u64::from(p.min_count),
            |_| format!("{example_count} examples"),
            || format!("{example_count} examples (need >= {min_examples})"),
        );
    }

    /// Evaluate the size and health of the contributor community.
    fn repo_contributor_count(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        let value = gh_data.contributors;

        let min_contributors = &self
            .config
            .repo_contributor_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::RepoContributorCount,
            &self.config.repo_contributor_count,
            |p| value >= u64::from(p.min_count),
            |_| format!("{value} contributors"),
            || format!("{value} contributors (need >= {min_contributors})"),
        );
    }

    /// Evaluate the number of repository stars.
    fn repo_star_count(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        let value = gh_data.stars;

        let min_stars = &self
            .config
            .repo_star_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::RepoStarCount,
            &self.config.repo_star_count,
            |p| value >= u64::from(p.min_count),
            |_| format!("{value} stars"),
            || format!("{value} stars (need >= {min_stars})"),
        );
    }

    /// Evaluate the number of repository forks.
    fn repo_fork_count(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        let value = gh_data.forks;

        let min_forks = &self
            .config
            .repo_fork_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::RepoForkCount,
            &self.config.repo_fork_count,
            |p| value >= u64::from(p.min_count),
            |_| format!("{value} forks"),
            || format!("{value} forks (need >= {min_forks})"),
        );
    }

    /// Evaluate the number of repository subscribers/watchers.
    fn repo_subscriber_count(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        let value = gh_data.subscribers;

        let min_subscribers = &self
            .config
            .repo_subscriber_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::RepoSubscriberCount,
            &self.config.repo_subscriber_count,
            |p| value >= u64::from(p.min_count),
            |_| format!("{value} subscribers"),
            || format!("{value} subscribers (need >= {min_subscribers})"),
        );
    }

    /// Evaluate recent commit activity in the repository.
    ///
    /// Note: Currently only supports a 90-day (3-month) time window. Policies with other
    /// time windows will result in a "not supported" message.
    fn commit_activity(&mut self) {
        const SUPPORTED_DAYS: u32 = 90;

        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        let commits = gh_data.commits_last_3_months;

        let min_commits = &self
            .config
            .commit_activity
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type) && p.max_days == SUPPORTED_DAYS)
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::CommitActivity,
            &self.config.commit_activity,
            |p| {
                if p.max_days != SUPPORTED_DAYS {
                    return false;
                }
                commits >= u64::from(p.min_count)
            },
            |p| format!("{commits} commits in last {} days", p.max_days),
            || format!("{commits} commits in last {SUPPORTED_DAYS} days (need >= {min_commits})"),
        );
    }

    /// Evaluate the number of open issues (fewer is better).
    fn open_issue_count(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        let value = gh_data.issues.open_count;

        let max_open_issues = &self
            .config
            .open_issue_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.max_count))
            .max()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::OpenIssueCount,
            &self.config.open_issue_count,
            |p| value <= u64::from(p.max_count),
            |_| format!("{value} open issues"),
            || format!("{value} open issues (need < {max_open_issues})"),
        );
    }

    /// Evaluate the number of closed issues (more is better).
    fn closed_issue_count(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        let value = gh_data.issues.closed_count;

        let min_closed_issues = &self
            .config
            .closed_issue_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::ClosedIssueCount,
            &self.config.closed_issue_count,
            |p| value >= u64::from(p.min_count),
            |_| format!("{value} closed issues"),
            || format!("{value} closed issues (need >= {min_closed_issues})"),
        );
    }

    /// Evaluate how quickly issues are addressed.
    fn issue_responsiveness(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        self.apply_responsiveness_policy(
            Metric::IssueResponsiveness,
            &self.config.issue_responsiveness,
            &gh_data.issues.closed_age,
        );
    }

    /// Evaluate the number of open pull requests (fewer is better).
    fn open_pull_request_count(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        let value = gh_data.pulls.open_count;

        let max_open_prs = &self
            .config
            .open_pull_request_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.max_count))
            .max()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::OpenPullRequestCount,
            &self.config.open_pull_request_count,
            |p| value <= u64::from(p.max_count),
            |_| format!("{value} open pull requests"),
            || format!("{value} open pull requests (need < {max_open_prs})"),
        );
    }

    /// Evaluate the number of closed pull requests (more is better).
    fn closed_pull_request_count(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        let value = gh_data.pulls.closed_count;

        let min_closed_prs = &self
            .config
            .closed_pull_request_count
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.min_count))
            .min()
            .unwrap_or(0);

        self.apply_generic_policy(
            Metric::ClosedPullRequestCount,
            &self.config.closed_pull_request_count,
            |p| value >= u64::from(p.min_count),
            |_| format!("{value} closed pull requests"),
            || format!("{value} closed pull requests (need >= {min_closed_prs})"),
        );
    }

    /// Evaluate how quickly pull requests are reviewed and merged.
    fn pull_request_responsiveness(&mut self) {
        let ProviderResult::Found(gh_data) = &self.facts.hosting_data else {
            return;
        };

        self.apply_responsiveness_policy(
            Metric::PullRequestResponsiveness,
            &self.config.pull_request_responsiveness,
            &gh_data.pulls.closed_age,
        );
    }

    /// Generic helper for policy evaluation with custom predicate.
    fn apply_generic_policy<T, P, S, F>(&mut self, metric: Metric, policies: &[T], predicate: P, success_msg_fn: S, failure_msg_fn: F)
    where
        T: Policy,
        P: Fn(&T) -> bool,
        S: Fn(&T) -> String,
        F: Fn() -> String,
    {
        let mut num_policies = 0;
        for policy in policies {
            if !policy.dependency_types().contains(self.dependency_type) {
                continue;
            }

            num_policies += 1;
            if predicate(policy) {
                self.add_matched(metric, self.scale_points(metric, policy.points()), success_msg_fn(policy));
                return;
            }
        }

        if num_policies == 0 {
            self.add_not_matched(metric, "no policy defined".to_string());
        } else {
            self.add_not_matched(metric, failure_msg_fn());
        }
    }

    /// Generic helper for responsiveness policies (checks all age percentile thresholds).
    fn apply_responsiveness_policy(&mut self, metric: Metric, policies: &[ResponsivenessPolicy], stats: &AgeStats) {
        for policy in policies {
            if !policy.dependency_types().contains(self.dependency_type) {
                continue;
            }

            if stats.avg <= policy.max_average_days
                && stats.p50 <= policy.max_p50_days
                && stats.p75 <= policy.max_p75_days
                && stats.p90 <= policy.max_p90_days
                && stats.p95 <= policy.max_p95_days
            {
                let points = self.scale_points(metric, policy.points());

                self.add_matched(metric, points, "sufficiently responsive".to_string());
                return;
            }
        }

        // If no policy matched, add a single NoMatch outcome
        self.add_not_matched(metric, "insufficiently responsive".to_string());
    }

    /// Scale a score by applying the metric's scale factor.
    /// If no scale factor is configured for the metric, defaults to 1.0 (no scaling).
    fn scale_points(&self, metric: Metric, points: f64) -> f64 {
        points * self.config.metric_scaling.get(&metric).copied().unwrap_or(1.0)
    }

    /// Add a matched policy result to the results map
    fn add_matched(&mut self, metric: Metric, points: f64, info: String) {
        _ = self.results.insert(metric, PolicyOutcome::Match(points, info));
    }

    /// Add a not matched policy result to the results map
    fn add_not_matched(&mut self, metric: Metric, reason: String) {
        _ = self.results.insert(metric, PolicyOutcome::NoMatch(reason));
    }

    /// Count the number of owners of a specific kind (Team or User).
    fn get_owner_count(&self, kind: OwnerKind) -> u64 {
        let ProviderResult::Found(crate_overall_data) = &self.facts.crate_overall_data else {
            unreachable!("analyzable crate must have Found data");
        };
        crate_overall_data
            .owners
            .iter()
            .filter(|x| x.kind == kind)
            .count()
            .try_into()
            .expect("owner count always fits in u64")
    }

    // Advisory metrics - version-specific

    /// Evaluate total vulnerability count for this version
    fn vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.vulnerability_count;
        let max_count = self.get_max_count(&self.config.vulnerability_count);

        self.apply_generic_policy(
            Metric::VulnerabilityCount,
            &self.config.vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} vulnerabilities"),
            || format!("{count} vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate low severity vulnerability count for this version
    fn low_vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.low_vulnerability_count;
        let max_count = self.get_max_count(&self.config.low_vulnerability_count);

        self.apply_generic_policy(
            Metric::LowVulnerabilityCount,
            &self.config.low_vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} low severity vulnerabilities"),
            || format!("{count} low severity vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate medium severity vulnerability count for this version
    fn medium_vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.medium_vulnerability_count;
        let max_count = self.get_max_count(&self.config.medium_vulnerability_count);

        self.apply_generic_policy(
            Metric::MediumVulnerabilityCount,
            &self.config.medium_vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} medium severity vulnerabilities"),
            || format!("{count} medium severity vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate high severity vulnerability count for this version
    fn high_vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.high_vulnerability_count;
        let max_count = self.get_max_count(&self.config.high_vulnerability_count);

        self.apply_generic_policy(
            Metric::HighVulnerabilityCount,
            &self.config.high_vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} high severity vulnerabilities"),
            || format!("{count} high severity vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate critical severity vulnerability count for this version
    fn critical_vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.critical_vulnerability_count;
        let max_count = self.get_max_count(&self.config.critical_vulnerability_count);

        self.apply_generic_policy(
            Metric::CriticalVulnerabilityCount,
            &self.config.critical_vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} critical severity vulnerabilities"),
            || format!("{count} critical severity vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate total warning count for this version
    fn warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.warning_count;
        let max_count = self.get_max_count(&self.config.warning_count);

        self.apply_generic_policy(
            Metric::WarningCount,
            &self.config.warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} warnings"),
            || format!("{count} warnings (need <= {max_count})"),
        );
    }

    /// Evaluate notice warning count for this version
    fn notice_warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.notice_warning_count;
        let max_count = self.get_max_count(&self.config.notice_warning_count);

        self.apply_generic_policy(
            Metric::NoticeWarningCount,
            &self.config.notice_warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} notice warnings"),
            || format!("{count} notice warnings (need <= {max_count})"),
        );
    }

    /// Evaluate unmaintained warning count for this version
    fn unmaintained_warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.unmaintained_warning_count;
        let max_count = self.get_max_count(&self.config.unmaintained_warning_count);

        self.apply_generic_policy(
            Metric::UnmaintainedWarningCount,
            &self.config.unmaintained_warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} unmaintained warnings"),
            || format!("{count} unmaintained warnings (need <= {max_count})"),
        );
    }

    /// Evaluate unsound warning count for this version
    fn unsound_warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.unsound_warning_count;
        let max_count = self.get_max_count(&self.config.unsound_warning_count);

        self.apply_generic_policy(
            Metric::UnsoundWarningCount,
            &self.config.unsound_warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} unsound warnings"),
            || format!("{count} unsound warnings (need <= {max_count})"),
        );
    }

    /// Evaluate yanked warning count for this version
    fn yanked_warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.yanked_warning_count;
        let max_count = self.get_max_count(&self.config.yanked_warning_count);

        self.apply_generic_policy(
            Metric::YankedWarningCount,
            &self.config.yanked_warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} yanked warnings"),
            || format!("{count} yanked warnings (need <= {max_count})"),
        );
    }

    // Advisory metrics - historical (all versions)

    /// Evaluate historical vulnerability count
    fn historical_vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_vulnerability_count;
        let max_count = self.get_max_count(&self.config.historical_vulnerability_count);

        self.apply_generic_policy(
            Metric::HistoricalVulnerabilityCount,
            &self.config.historical_vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical vulnerabilities"),
            || format!("{count} historical vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate historical low severity vulnerability count
    fn historical_low_vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_low_vulnerability_count;
        let max_count = self.get_max_count(&self.config.historical_low_vulnerability_count);

        self.apply_generic_policy(
            Metric::HistoricalLowVulnerabilityCount,
            &self.config.historical_low_vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical low severity vulnerabilities"),
            || format!("{count} historical low severity vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate historical medium severity vulnerability count
    fn historical_medium_vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_medium_vulnerability_count;
        let max_count = self.get_max_count(&self.config.historical_medium_vulnerability_count);

        self.apply_generic_policy(
            Metric::HistoricalMediumVulnerabilityCount,
            &self.config.historical_medium_vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical medium severity vulnerabilities"),
            || format!("{count} historical medium severity vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate historical high severity vulnerability count
    fn historical_high_vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_high_vulnerability_count;
        let max_count = self.get_max_count(&self.config.historical_high_vulnerability_count);

        self.apply_generic_policy(
            Metric::HistoricalHighVulnerabilityCount,
            &self.config.historical_high_vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical high severity vulnerabilities"),
            || format!("{count} historical high severity vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate historical critical severity vulnerability count
    fn historical_critical_vulnerability_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_critical_vulnerability_count;
        let max_count = self.get_max_count(&self.config.historical_critical_vulnerability_count);

        self.apply_generic_policy(
            Metric::HistoricalCriticalVulnerabilityCount,
            &self.config.historical_critical_vulnerability_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical critical severity vulnerabilities"),
            || format!("{count} historical critical severity vulnerabilities (need <= {max_count})"),
        );
    }

    /// Evaluate historical warning count
    fn historical_warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_warning_count;
        let max_count = self.get_max_count(&self.config.historical_warning_count);

        self.apply_generic_policy(
            Metric::HistoricalWarningCount,
            &self.config.historical_warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical warnings"),
            || format!("{count} historical warnings (need <= {max_count})"),
        );
    }

    /// Evaluate historical notice warning count
    fn historical_notice_warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_notice_warning_count;
        let max_count = self.get_max_count(&self.config.historical_notice_warning_count);

        self.apply_generic_policy(
            Metric::HistoricalNoticeWarningCount,
            &self.config.historical_notice_warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical notice warnings"),
            || format!("{count} historical notice warnings (need <= {max_count})"),
        );
    }

    /// Evaluate historical unmaintained warning count
    fn historical_unmaintained_warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_unmaintained_warning_count;
        let max_count = self.get_max_count(&self.config.historical_unmaintained_warning_count);

        self.apply_generic_policy(
            Metric::HistoricalUnmaintainedWarningCount,
            &self.config.historical_unmaintained_warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical unmaintained warnings"),
            || format!("{count} historical unmaintained warnings (need <= {max_count})"),
        );
    }

    /// Evaluate historical unsound warning count
    fn historical_unsound_warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_unsound_warning_count;
        let max_count = self.get_max_count(&self.config.historical_unsound_warning_count);

        self.apply_generic_policy(
            Metric::HistoricalUnsoundWarningCount,
            &self.config.historical_unsound_warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical unsound warnings"),
            || format!("{count} historical unsound warnings (need <= {max_count})"),
        );
    }

    /// Evaluate historical yanked warning count
    fn historical_yanked_warning_count(&mut self) {
        let ProviderResult::Found(advisory_data) = &self.facts.advisory_data else {
            return;
        };
        let count = advisory_data.historical_yanked_warning_count;
        let max_count = self.get_max_count(&self.config.historical_yanked_warning_count);

        self.apply_generic_policy(
            Metric::HistoricalYankedWarningCount,
            &self.config.historical_yanked_warning_count,
            |p| count <= u64::from(p.max_count),
            |_| format!("{count} historical yanked warnings"),
            || format!("{count} historical yanked warnings (need <= {max_count})"),
        );
    }

    /// Helper to get the maximum count from a `MaxCountPolicy` vector
    fn get_max_count(&self, policies: &[crate::config::MaxCountPolicy]) -> u64 {
        policies
            .iter()
            .filter(|p| p.dependency_types().contains(self.dependency_type))
            .map(|p| u64::from(p.max_count))
            .max()
            .unwrap_or(0)
    }
}

/// Check if the current version was released within the specified number of days.
/// Returns 1 if yes, 0 if no.
fn count_releases_in_period(facts: &CrateFacts, days: u32) -> usize {
    let cutoff = Utc::now() - Duration::days(i64::from(days));
    let ProviderResult::Found(crate_version_data) = &facts.crate_version_data else {
        unreachable!("analyzable crate must have Found data");
    };
    usize::from(crate_version_data.created_at >= cutoff)
}
