//! Building one half of a run into a single encrypted archive.
//!
//! The pipeline is `tar -> zstd -> age`, composed as nested writers so the data is
//! streamed through in one pass rather than staged three times on disk. Order is not
//! negotiable: age output is indistinguishable from random, so compressing after
//! encrypting would inflate the archive instead of shrinking it.

use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;

use crate::crypt::{self, Recipient};
use crate::plan::Plan;

/// Default zstd level. 3 is the zstd default: most of the ratio, a fraction of the CPU of
/// the higher levels, which matters on a node that is also serving the cluster.
pub const DEFAULT_LEVEL: i32 = 3;

/// What a completed archive turned out to be.
#[derive(Debug, Clone, Copy)]
pub struct ArchiveStats {
    /// Size on disk of the finished, encrypted archive.
    pub bytes: u64,
    /// Number of top-level entries included.
    pub members: usize,
}

/// Archive `plan`'s members from `run` into `dest`, compressed and encrypted.
///
/// `dest` is created with mode 0600 before any bytes are written: even encrypted, this
/// file is the whole cluster, and it should never be briefly world-readable.
pub fn build(
    run: &Path,
    plan: &Plan,
    recipient: &Recipient,
    dest: &Path,
    level: i32,
) -> Result<ArchiveStats> {
    let file =
        File::create(dest).with_context(|| format!("creating archive {}", dest.display()))?;
    restrict(&file, dest)?;

    // file <- age <- zstd <- tar
    let age_w = crypt::wrap(recipient, file)?;
    let mut zstd_w = zstd::Encoder::new(age_w, level).context("starting zstd encoder")?;
    // Best-effort: on a single-core box this is a no-op rather than an error.
    let _ = zstd_w.multithread(num_workers());

    {
        let mut tar = tar::Builder::new(&mut zstd_w);
        // Directory entries carry no useful ownership for a restore that runs as root and
        // re-chowns anyway, but they do carry the mtimes, which are worth keeping.
        for member in &plan.members {
            let path = run.join(member);
            if path.is_dir() {
                tar.append_dir_all(member, &path)
                    .with_context(|| format!("archiving directory {member}"))?;
            } else {
                tar.append_path_with_name(&path, member)
                    .with_context(|| format!("archiving file {member}"))?;
            }
        }
        tar.finish().context("finishing tar")?;
    }

    let age_w = zstd_w.finish().context("finishing zstd stream")?;
    let file = crypt::finish(age_w)?;
    // Without this the process can exit reporting success while the tail of the archive is
    // still only in the page cache.
    file.sync_all().context("fsync archive")?;

    let bytes = file.metadata().context("stat finished archive")?.len();

    Ok(ArchiveStats {
        bytes,
        members: plan.members.len(),
    })
}

fn num_workers() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get().min(4) as u32)
        .unwrap_or(1)
}

#[cfg(unix)]
fn restrict(file: &File, dest: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("chmod 600 {}", dest.display()))
}

#[cfg(not(unix))]
fn restrict(_file: &File, _dest: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::Half;
    use std::fs;
    use std::io::Read;

    #[test]
    fn archive_round_trips_back_to_the_original_files() {
        let run = tempfile::tempdir().expect("run dir");
        fs::write(
            run.path().join("MANIFEST.txt"),
            b"cluster backup 20260901-120000",
        )
        .unwrap();
        fs::write(run.path().join("sealed-secrets-key.yaml"), b"MASTER-KEY").unwrap();
        fs::create_dir(run.path().join("secrets")).unwrap();
        fs::write(run.path().join("secrets/a.yaml"), b"secret-a").unwrap();

        let identity = age::x25519::Identity::generate();
        let recipient = Recipient::parse(&identity.to_public().to_string()).unwrap();

        let out = tempfile::tempdir().expect("out dir");
        let dest = out.path().join("critical.tar.zst.age");
        let plan = Plan::resolve(run.path(), Half::Critical);
        let stats = build(run.path(), &plan, &recipient, &dest, DEFAULT_LEVEL).expect("build");

        assert_eq!(stats.members, 3, "manifest, key and secrets/");
        assert!(stats.bytes > 0);
        assert_eq!(stats.bytes, fs::metadata(&dest).unwrap().len());

        // Decrypt -> decompress -> untar, and confirm the bytes survived the whole pipeline.
        let ciphertext = fs::read(&dest).unwrap();
        let decryptor = age::Decryptor::new(&ciphertext[..]).expect("age header");
        let reader = decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .expect("decrypt");
        let zstd_r = zstd::Decoder::new(reader).expect("zstd");
        let mut tar = tar::Archive::new(zstd_r);

        let mut found = std::collections::BTreeMap::new();
        for entry in tar.entries().expect("entries") {
            let mut e = entry.expect("entry");
            let path = e.path().expect("path").to_string_lossy().into_owned();
            let mut buf = Vec::new();
            e.read_to_end(&mut buf).expect("read entry");
            found.insert(path, buf);
        }

        assert_eq!(
            found.get("sealed-secrets-key.yaml").map(|v| v.as_slice()),
            Some(&b"MASTER-KEY"[..])
        );
        assert_eq!(
            found.get("secrets/a.yaml").map(|v| v.as_slice()),
            Some(&b"secret-a"[..])
        );
        assert!(found.contains_key("MANIFEST.txt"));
    }

    #[cfg(unix)]
    #[test]
    fn archive_is_not_world_readable() {
        use std::os::unix::fs::PermissionsExt;
        let run = tempfile::tempdir().unwrap();
        fs::write(run.path().join("MANIFEST.txt"), b"m").unwrap();
        let identity = age::x25519::Identity::generate();
        let recipient = Recipient::parse(&identity.to_public().to_string()).unwrap();
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("a.age");

        let plan = Plan::resolve(run.path(), Half::Critical);
        build(run.path(), &plan, &recipient, &dest, DEFAULT_LEVEL).unwrap();

        let mode = fs::metadata(&dest).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "archive must be owner-only, got {mode:o}");
    }
}
