//! Deciding what goes where.
//!
//! A backup run is split into two halves that are stored very differently, because
//! Glacier Deep Archive takes 12-48 hours to restore. Putting the Sealed-Secrets master
//! key and the microk8s CA there means a cluster rebuild cannot *start* for up to two
//! days. Those pieces total tens of kilobytes, so keeping them instantly retrievable is
//! effectively free.

use std::path::Path;

/// Top-level entries archived into the critical half, in priority order.
///
/// These are the pieces a rebuild needs in its first five minutes: without the master key
/// every committed `SealedSecret` is undecryptable, and without `ca.key` every kubeconfig
/// ever minted from the cluster is invalid.
pub const CRITICAL_MEMBERS: &[&str] =
    &["MANIFEST.txt", "sealed-secrets-key.yaml", "secrets", "node"];

/// Top-level entries archived into the bulk half.
///
/// Database dumps and volume tarballs: large, and useless until the cluster is already
/// standing back up, which is exactly the workload Deep Archive is priced for.
pub const BULK_MEMBERS: &[&str] = &["db", "data"];

/// Which half of a run an archive belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Half {
    /// Small, restore-blocking material. Stored for instant retrieval.
    Critical,
    /// Large, restore-deferred material. Stored cold.
    Bulk,
}

impl Half {
    /// The S3 key component this half is filed under.
    pub fn as_str(self) -> &'static str {
        match self {
            Half::Critical => "critical",
            Half::Bulk => "bulk",
        }
    }

    /// The entries this half would archive, if present in the run.
    pub fn members(self) -> &'static [&'static str] {
        match self {
            Half::Critical => CRITICAL_MEMBERS,
            Half::Bulk => BULK_MEMBERS,
        }
    }
}

/// The entries of one half that actually exist in a given run directory.
///
/// Membership is resolved against the filesystem rather than assumed: `sealed-secrets-key.yaml`
/// is absent whenever the controller is not installed, and `tar` aborts outright on a member
/// that does not exist.
#[derive(Debug, Clone)]
pub struct Plan {
    /// Which half this plan covers.
    pub half: Half,
    /// Entry names present in the run, relative to its root.
    pub members: Vec<String>,
}

impl Plan {
    /// Resolve the members of `half` that exist under `run`.
    pub fn resolve(run: &Path, half: Half) -> Self {
        let members = half
            .members()
            .iter()
            .filter(|m| run.join(m).exists())
            .map(|m| (*m).to_string())
            .collect();
        Plan { half, members }
    }

    /// True when there is nothing to archive for this half.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

/// Whether a run directory is a completed backup.
///
/// `MANIFEST.txt` is written last, so its presence is the only honest marker that the
/// collection step ran to completion. Shipping a partial run is worse than shipping
/// nothing: it looks like a backup and is not one.
pub fn is_complete(run: &Path) -> bool {
    run.join("MANIFEST.txt").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn run_with(entries: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for e in entries {
            let p = dir.path().join(e);
            if e.contains('.') {
                fs::write(&p, b"x").expect("write");
            } else {
                fs::create_dir_all(&p).expect("mkdir");
            }
        }
        dir
    }

    #[test]
    fn resolves_only_entries_that_exist() {
        let dir = run_with(&["MANIFEST.txt", "secrets", "db"]);
        let crit = Plan::resolve(dir.path(), Half::Critical);
        // `sealed-secrets-key.yaml` and `node` are absent and must not be listed, or tar fails.
        assert_eq!(crit.members, vec!["MANIFEST.txt", "secrets"]);

        let bulk = Plan::resolve(dir.path(), Half::Bulk);
        assert_eq!(bulk.members, vec!["db"]);
    }

    #[test]
    fn empty_half_is_reported_not_silently_archived() {
        let dir = run_with(&["MANIFEST.txt"]);
        assert!(Plan::resolve(dir.path(), Half::Bulk).is_empty());
        assert!(!Plan::resolve(dir.path(), Half::Critical).is_empty());
    }

    #[test]
    fn completeness_tracks_the_manifest_only() {
        let partial = run_with(&["secrets", "db"]);
        assert!(!is_complete(partial.path()), "no manifest means incomplete");

        let done = run_with(&["MANIFEST.txt", "secrets"]);
        assert!(is_complete(done.path()));
    }

    #[test]
    fn halves_do_not_overlap() {
        for c in CRITICAL_MEMBERS {
            assert!(!BULK_MEMBERS.contains(c), "{c} is in both halves");
        }
    }
}
