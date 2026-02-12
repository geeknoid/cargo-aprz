use super::CrateSpec;
use core::fmt::{Display, Formatter, Result as FmtResult};
use core::str::FromStr;
use semver::Version;
use std::sync::Arc;
use ohno::IntoAppError;

/// A crate identifier consisting of a name and optional version
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrateRef {
    name: Arc<str>,
    version: Option<Arc<Version>>,
}

impl CrateRef {
    /// Create a new crate ID with name and optional version
    #[must_use]
    pub fn new(name: impl AsRef<str>, version: Option<Version>) -> Self {
        Self {
            name: Arc::from(name.as_ref()),
            version: version.map(Arc::new),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn version(&self) -> Option<&Version> {
        self.version.as_deref()
    }

    /// Get a clone of the name Arc
    #[must_use]
    pub fn name_arc(&self) -> Arc<str> {
        Arc::clone(&self.name)
    }

    /// Get a clone of the version Arc if present (cheap pointer clone, no Version allocation)
    #[must_use]
    pub fn version_arc(&self) -> Option<Arc<Version>> {
        self.version.as_ref().map(Arc::clone)
    }

    /// Convert to a `CrateSpec` by cloning Arc pointers (no allocation)
    #[must_use]
    pub fn to_spec(&self) -> Option<CrateSpec> {
        Some(CrateSpec::from_arcs(Arc::clone(&self.name), Arc::clone(self.version.as_ref()?)))
    }
}

impl FromStr for CrateRef {
    type Err = ohno::AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((name, version_str)) = s.split_once('@') {
            let version =
                Version::parse(version_str).into_app_err_with(|| format!("parsing version '{version_str}' in crate specifier '{s}'"))?;
            Ok(Self::new(name, Some(version)))
        } else {
            Ok(Self::new(s, None))
        }
    }
}

impl Display for CrateRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.name())?;
        if let Some(version) = self.version() {
            write!(f, "@{version}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    // --- Construction ---

    #[test]
    fn test_new_with_version() {
        let v = Version::parse("1.2.3").unwrap();
        let cr = CrateRef::new("serde", Some(v.clone()));
        assert_eq!(cr.name(), "serde");
        assert_eq!(cr.version(), Some(&v));
    }

    #[test]
    fn test_new_without_version() {
        let cr = CrateRef::new("tokio", None);
        assert_eq!(cr.name(), "tokio");
        assert_eq!(cr.version(), None);
    }

    #[test]
    fn test_new_accepts_string() {
        let cr = CrateRef::new(String::from("anyhow"), None);
        assert_eq!(cr.name(), "anyhow");
    }

    // --- Arc accessors ---

    #[test]
    fn test_name_arc() {
        let cr = CrateRef::new("serde", None);
        let arc = cr.name_arc();
        assert_eq!(&*arc, "serde");
        // Verify it's a cheap clone (same pointer)
        let arc2 = cr.name_arc();
        assert!(Arc::ptr_eq(&arc, &arc2));
    }

    #[test]
    fn test_version_arc_some() {
        let v = Version::parse("0.1.0").unwrap();
        let cr = CrateRef::new("foo", Some(v));
        let arc = cr.version_arc();
        assert!(arc.is_some());
        assert_eq!(arc.as_deref(), Some(&Version::parse("0.1.0").unwrap()));
    }

    #[test]
    fn test_version_arc_none() {
        let cr = CrateRef::new("foo", None);
        assert!(cr.version_arc().is_none());
    }

    // --- to_spec ---

    #[test]
    fn test_to_spec_with_version() {
        let v = Version::parse("2.0.0").unwrap();
        let cr = CrateRef::new("hyper", Some(v));
        let spec = cr.to_spec();
        assert!(spec.is_some());
        let spec = spec.unwrap();
        assert_eq!(spec.name(), "hyper");
        assert_eq!(*spec.version(), Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn test_to_spec_without_version() {
        let cr = CrateRef::new("hyper", None);
        assert!(cr.to_spec().is_none());
    }

    // --- FromStr ---

    #[test]
    fn test_from_str_with_version() {
        let cr: CrateRef = "serde@1.0.200".parse().unwrap();
        assert_eq!(cr.name(), "serde");
        assert_eq!(cr.version(), Some(&Version::parse("1.0.200").unwrap()));
    }

    #[test]
    fn test_from_str_without_version() {
        let cr: CrateRef = "tokio".parse().unwrap();
        assert_eq!(cr.name(), "tokio");
        assert_eq!(cr.version(), None);
    }

    #[test]
    fn test_from_str_with_prerelease_version() {
        let cr: CrateRef = "my-crate@0.1.0-alpha.1".parse().unwrap();
        assert_eq!(cr.name(), "my-crate");
        assert_eq!(cr.version(), Some(&Version::parse("0.1.0-alpha.1").unwrap()));
    }

    #[test]
    fn test_from_str_with_build_metadata() {
        let cr: CrateRef = "foo@1.0.0+build.42".parse().unwrap();
        assert_eq!(cr.name(), "foo");
        assert_eq!(cr.version(), Some(&Version::parse("1.0.0+build.42").unwrap()));
    }

    #[test]
    fn test_from_str_invalid_version() {
        let result: Result<CrateRef, _> = "bad@notaversion".parse();
        let _ = result.unwrap_err();
    }

    // --- Display ---

    #[test]
    fn test_display_with_version() {
        let cr = CrateRef::new("serde", Some(Version::parse("1.0.200").unwrap()));
        assert_eq!(cr.to_string(), "serde@1.0.200");
    }

    #[test]
    fn test_display_without_version() {
        let cr = CrateRef::new("tokio", None);
        assert_eq!(cr.to_string(), "tokio");
    }

    #[test]
    fn test_display_with_prerelease() {
        let cr = CrateRef::new("x", Some(Version::parse("0.1.0-beta.2").unwrap()));
        assert_eq!(cr.to_string(), "x@0.1.0-beta.2");
    }

    // --- Derived traits ---

    #[test]
    fn test_clone() {
        let cr = CrateRef::new("serde", Some(Version::parse("1.0.0").unwrap()));
        let cr2 = cr.clone();
        assert_eq!(cr, cr2);
    }

    #[test]
    fn test_eq_same() {
        let a = CrateRef::new("a", Some(Version::parse("1.0.0").unwrap()));
        let b = CrateRef::new("a", Some(Version::parse("1.0.0").unwrap()));
        assert_eq!(a, b);
    }

    #[test]
    fn test_eq_both_no_version() {
        let a = CrateRef::new("a", None);
        let b = CrateRef::new("a", None);
        assert_eq!(a, b);
    }

    #[test]
    fn test_ne_different_name() {
        let a = CrateRef::new("a", None);
        let b = CrateRef::new("b", None);
        assert_ne!(a, b);
    }

    #[test]
    fn test_ne_different_version() {
        let a = CrateRef::new("a", Some(Version::parse("1.0.0").unwrap()));
        let b = CrateRef::new("a", Some(Version::parse("2.0.0").unwrap()));
        assert_ne!(a, b);
    }

    #[test]
    fn test_ne_version_vs_none() {
        let a = CrateRef::new("a", Some(Version::parse("1.0.0").unwrap()));
        let b = CrateRef::new("a", None);
        assert_ne!(a, b);
    }

    #[test]
    fn test_hash_equal_values_produce_equal_hashes() {
        use std::collections::hash_map::DefaultHasher;
        use core::hash::{Hash, Hasher};

        let a = CrateRef::new("x", Some(Version::parse("1.0.0").unwrap()));
        let b = CrateRef::new("x", Some(Version::parse("1.0.0").unwrap()));

        let hash = |cr: &CrateRef| {
            let mut h = DefaultHasher::new();
            cr.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash(&a), hash(&b));
    }

    #[test]
    fn test_hash_usable_in_hashset() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let _ = set.insert(CrateRef::new("a", Some(Version::parse("1.0.0").unwrap())));
        let _ = set.insert(CrateRef::new("a", Some(Version::parse("1.0.0").unwrap())));
        let _ = set.insert(CrateRef::new("a", None));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_debug() {
        let cr = CrateRef::new("serde", Some(Version::parse("1.0.0").unwrap()));
        let debug = format!("{cr:?}");
        assert!(debug.contains("serde"));
        assert!(debug.contains("CrateRef"));
    }

    // --- FromStr / Display round-trip ---

    #[test]
    fn test_display_from_str_roundtrip_with_version() {
        let cr = CrateRef::new("serde", Some(Version::parse("1.0.200").unwrap()));
        let parsed: CrateRef = cr.to_string().parse().unwrap();
        assert_eq!(cr, parsed);
    }

    #[test]
    fn test_display_from_str_roundtrip_without_version() {
        let cr = CrateRef::new("tokio", None);
        let parsed: CrateRef = cr.to_string().parse().unwrap();
        assert_eq!(cr, parsed);
    }
}
