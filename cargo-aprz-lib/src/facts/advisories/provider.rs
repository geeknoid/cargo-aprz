use super::AdvisoryData;
use crate::Result;
use crate::facts::ProviderResult;
use crate::facts::cache::{Cache, CacheResult};
use crate::facts::crate_spec::CrateSpec;
use crate::facts::progress::Progress;
use compact_str::CompactString;
use core::time::Duration;
use ohno::IntoAppError;
use rustsec::{
    database::Database,
    repository::git::{DEFAULT_URL, Repository},
};
use crate::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Log target for advisories provider
const LOG_TARGET: &str = "advisories";

#[derive(Debug)]
pub struct Provider {
    database: Arc<Database>,
}

const DATABASE_FETCH_TIMEOUT: Duration = Duration::from_secs(60);

impl Provider {
    pub async fn new(
        cache: &Cache,
        progress: Arc<dyn Progress>,
    ) -> Result<Self> {
        let cache_dir = cache.dir();
        let sync_filename = "last_synced.json";
        let repo_path = cache_dir.join("repo");

        let needs_fetch = matches!(cache.load::<()>(sync_filename), CacheResult::Miss);

        if needs_fetch {
            download_db(&repo_path, progress.as_ref())
                .await
                .into_app_err("downloading the advisory database")?;
            cache.save(sync_filename, &())?;
        }

        Ok(Self {
            database: Arc::new(open_db(&repo_path, progress.as_ref()).await.into_app_err("opening the advisory database")?),
        })
    }

    pub async fn get_advisory_data(
        &self,
        crates: impl IntoIterator<Item = CrateSpec> + Send + 'static,
    ) -> impl Iterator<Item = (CrateSpec, ProviderResult<AdvisoryData>)> {
        let database = Arc::clone(&self.database);

        tokio::task::spawn_blocking(move || scan_advisories(&database, crates))
            .await
            .expect("tasks must not panic")
    }
}

fn scan_advisories<I>(
    database: &Database,
    crates: I,
) -> impl Iterator<Item = (CrateSpec, ProviderResult<AdvisoryData>)> + use<I>
where
    I: IntoIterator<Item = CrateSpec>,
{
    let start_time = std::time::Instant::now();

    let mut crate_map: HashMap<CompactString, Vec<(CrateSpec, ProviderResult<AdvisoryData>)>> = HashMap::default();

    for crate_spec in crates {
        crate_map.entry(crate_spec.name().into()).or_default().push((
            crate_spec,
            ProviderResult::Found(AdvisoryData::default()),
        ));
    }

    let crate_count = crate_map.len();
    let mut advisories_checked = 0;
    let mut advisories_matched = 0;

    log::info!(target: LOG_TARGET, "Querying the advisory database for {crate_count} crate(s)");

    for advisory in database.iter() {
        advisories_checked += 1;

        if let Some(crate_entries) = crate_map.get_mut(advisory.metadata.package.as_str()) {
            for (crate_spec, result) in crate_entries.iter_mut() {
                advisories_matched += 1;

                if let ProviderResult::Found(data) = result {
                    data.count_advisory_historical(advisory);
                    if advisory.versions.is_vulnerable(crate_spec.version()) {
                        data.count_advisory_for_version(advisory);
                    }
                }
            }
        }
    }

    log::debug!(
        target: LOG_TARGET,
        "Completed scan of advisory database: checked {} advisories, found {} matches for {} crates in {:.3}s",
        advisories_checked,
        advisories_matched,
        crate_count,
        start_time.elapsed().as_secs_f64()
    );

    crate_map.into_values().flatten()
}

async fn open_db(cache_dir: impl AsRef<Path>, progress: &dyn Progress) -> Result<Database> {
    let cache_path = cache_dir.as_ref().to_path_buf();

    run_blocking_with_progress(
        progress,
        "Opening the advisory database",
        "opening",
        move || Database::open(&cache_path).map_err(Into::into),
    )
    .await
}

async fn download_db(cache_dir: impl AsRef<Path>, progress: &dyn Progress) -> Result<()> {
    let cache_path = cache_dir.as_ref().to_path_buf();

    run_blocking_with_progress(
        progress,
        "Downloading the advisory database",
        "downloading",
        move || {
            Repository::fetch(DEFAULT_URL, &cache_path, true, DATABASE_FETCH_TIMEOUT)
                .map(|_| ())
                .map_err(Into::into)
        },
    )
    .await
}

async fn run_blocking_with_progress<T, F>(
    progress: &dyn Progress,
    msg: &str,
    success_verb: &str,
    blocking_fn: F,
) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    log::info!(target: LOG_TARGET, "{msg}");

    let progress_msg = msg.to_string();
    let start_time = std::time::Instant::now();
    progress.set_indeterminate(Box::new(move || progress_msg.clone()));

    let result = tokio::task::spawn_blocking(blocking_fn).await??;

    let elapsed = start_time.elapsed();
    log::debug!(target: LOG_TARGET, "Finished {success_verb} the advisory database in {:.3}s", elapsed.as_secs_f64());
    Ok(result)
}
