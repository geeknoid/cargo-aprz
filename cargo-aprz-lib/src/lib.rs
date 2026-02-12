#![doc(hidden)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

//! This is an implementation detail of the cargo-aprz tool. Do not take a dependency on this crate
//! as it may change in incompatible ways without warning.

// Core library for cargo-aprz
//
// This library consolidates all functionality for the cargo-aprz tool, which analyzes
// Rust crates for compliance with user-defined policies.
//
// # Module Organization
//
// - [`commands`]: Command-line interface and orchestration
// - [`facts`]: Data collection and aggregation
// - [`metrics`]: Metric extraction from facts
// - [`expr`]: Expression-based evaluation
// - [`reports`]: Report generation in multiple formats

pub type Result<T, E = ohno::AppError> = core::result::Result<T, E>;
pub(crate) type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;
pub(crate) type HashSet<V> = rustc_hash::FxHashSet<V>;

pub(crate) fn hash_map_with_capacity<K, V>(capacity: usize) -> HashMap<K, V> {
    HashMap::with_capacity_and_hasher(capacity, rustc_hash::FxBuildHasher)
}

pub(crate) fn hash_set_with_capacity<V>(capacity: usize) -> HashSet<V> {
    HashSet::with_capacity_and_hasher(capacity, rustc_hash::FxBuildHasher)
}

macro_rules! declare_modules {
    ($($mod:ident),+ $(,)?) => {
        $(
            #[cfg(debug_assertions)]
            pub mod $mod;
            #[cfg(not(debug_assertions))]
            mod $mod;
        )+
    };
}

declare_modules!(commands, expr, facts, metrics, reports);

pub use crate::commands::{Host, run};
