//! Archive, encrypt and ship a microk8s backup run offsite.
//!
//! A *run* is one timestamped directory produced by the collection step — `20260901-093329/`
//! holding `db/`, `data/`, `secrets/`, `node/`, `sealed-secrets-key.yaml` and `MANIFEST.txt`.
//! This crate takes such a directory and puts it in S3, in two pieces:
//!
//! | half | contents | storage class |
//! |------|----------|---------------|
//! | [`plan::Half::Critical`] | master key, CA, netplan, NetBird, secrets, manifest | instantly retrievable |
//! | [`plan::Half::Bulk`] | database dumps, volume tarballs | cold |
//!
//! The split exists because Glacier Deep Archive takes 12–48 hours to restore. The critical
//! half is tens of kilobytes; keeping it warm costs almost nothing and is the difference
//! between a rebuild starting now and starting the day after tomorrow.
//!
//! Everything is encrypted to an age recipient before it leaves the host, which holds only
//! the public key — see [`crypt`].

pub mod archive;
pub mod crypt;
pub mod plan;
pub mod s3;
