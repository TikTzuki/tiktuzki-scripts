//! Shipping an archive to S3 and proving it arrived.

use anyhow::{Context, Result, bail};
use aws_config::{BehaviorVersion, Region, defaults};
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::{ByteStream, Length};
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart, StorageClass};
use log::{info, warn};
use std::path::Path;

/// Files at or above this size are uploaded in parts.
///
/// A single `PutObject` is capped at 5 GiB, and a failure at 4.9 GiB restarts from zero.
/// Multipart also lets the SDK retry an individual part.
pub const MULTIPART_THRESHOLD: u64 = 100 * 1024 * 1024;

/// Part size for multipart uploads. 64 MiB against the 10,000-part cap allows ~640 GiB,
/// far beyond anything this cluster will produce.
pub const PART_SIZE: u64 = 64 * 1024 * 1024;

/// What S3 reports back about a stored object.
#[derive(Debug, Clone)]
pub struct StoredObject {
    /// Size S3 recorded.
    pub bytes: u64,
    /// Storage class S3 recorded. `STANDARD` is normalised in from the API's `None`.
    pub storage_class: String,
}

/// An S3 destination for backup archives.
#[derive(Debug, Clone)]
pub struct Uploader {
    client: Client,
    bucket: String,
}

impl Uploader {
    /// Build a client from the ambient AWS configuration.
    ///
    /// Credentials come from the standard chain, so on node1 this is an IAM user whose
    /// policy should allow `s3:PutObject` and nothing else — a host that cannot delete or
    /// read its own backups cannot be made to destroy them.
    pub async fn new(bucket: impl Into<String>, region: Option<String>) -> Result<Self> {
        let mut loader = defaults(BehaviorVersion::v2025_08_07());
        if let Some(r) = region {
            loader = loader.region(Region::new(r));
        }
        let config = loader.load().await;
        if config.region().is_none() {
            bail!("no AWS region configured; pass --region or set AWS_REGION");
        }
        Ok(Self {
            client: Client::new(&config),
            bucket: bucket.into(),
        })
    }

    /// Upload `path` to `key`, then confirm it is really there at the expected size.
    ///
    /// A successful `PutObject` only means the request was accepted; the `head_object`
    /// afterwards is what turns that into evidence.
    pub async fn upload_verified(
        &self,
        key: &str,
        path: &Path,
        class: &StorageClass,
    ) -> Result<StoredObject> {
        let len = tokio::fs::metadata(path)
            .await
            .with_context(|| format!("stat {}", path.display()))?
            .len();
        if len == 0 {
            bail!("refusing to upload an empty archive: {}", path.display());
        }

        if len >= MULTIPART_THRESHOLD {
            self.put_multipart(key, path, len, class).await?;
        } else {
            self.put_single(key, path, class).await?;
        }

        let stored = self.head(key).await?;
        if stored.bytes != len {
            bail!(
                "size mismatch for s3://{}/{}: sent {len}, stored {}",
                self.bucket,
                key,
                stored.bytes
            );
        }
        let want = class.as_str();
        if stored.storage_class != want {
            // Not fatal: a bucket lifecycle rule legitimately re-files objects, and the
            // data is intact either way. But it silently changes retrieval time and cost.
            warn!(
                "s3://{}/{} stored as {} but {} was requested — check the bucket lifecycle rules",
                self.bucket, key, stored.storage_class, want
            );
        }
        Ok(stored)
    }

    async fn put_single(&self, key: &str, path: &Path, class: &StorageClass) -> Result<()> {
        info!("put s3://{}/{} [{}]", self.bucket, key, class.as_str());
        let body = ByteStream::read_from()
            .path(path)
            .build()
            .await
            .with_context(|| format!("reading {}", path.display()))?;
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .storage_class(class.clone())
            .body(body)
            .send()
            .await
            .with_context(|| format!("put_object s3://{}/{key}", self.bucket))?;
        Ok(())
    }

    async fn put_multipart(
        &self,
        key: &str,
        path: &Path,
        len: u64,
        class: &StorageClass,
    ) -> Result<()> {
        let parts = len.div_ceil(PART_SIZE);
        info!(
            "multipart put s3://{}/{} [{}] — {len} bytes in {parts} parts",
            self.bucket,
            key,
            class.as_str()
        );

        let created = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .storage_class(class.clone())
            .send()
            .await
            .with_context(|| format!("create_multipart_upload s3://{}/{key}", self.bucket))?;
        let upload_id = created
            .upload_id()
            .context("S3 accepted the multipart upload but returned no upload id")?
            .to_string();

        // Abort on any failure. Without this, failed parts linger and are billed until a
        // lifecycle rule reaps them — and most buckets have no such rule.
        match self.upload_parts(key, path, len, parts, &upload_id).await {
            Ok(completed) => {
                self.client
                    .complete_multipart_upload()
                    .bucket(&self.bucket)
                    .key(key)
                    .upload_id(&upload_id)
                    .multipart_upload(
                        CompletedMultipartUpload::builder()
                            .set_parts(Some(completed))
                            .build(),
                    )
                    .send()
                    .await
                    .with_context(|| {
                        format!("complete_multipart_upload s3://{}/{key}", self.bucket)
                    })?;
                Ok(())
            }
            Err(e) => {
                warn!("aborting multipart upload {upload_id} after failure");
                if let Err(abort_err) = self
                    .client
                    .abort_multipart_upload()
                    .bucket(&self.bucket)
                    .key(key)
                    .upload_id(&upload_id)
                    .send()
                    .await
                {
                    warn!("abort also failed ({abort_err}); orphaned parts may be billed");
                }
                Err(e)
            }
        }
    }

    async fn upload_parts(
        &self,
        key: &str,
        path: &Path,
        len: u64,
        parts: u64,
        upload_id: &str,
    ) -> Result<Vec<CompletedPart>> {
        let mut completed = Vec::with_capacity(parts as usize);
        for i in 0..parts {
            let offset = i * PART_SIZE;
            let this = PART_SIZE.min(len - offset);
            // Part numbers are 1-based; a 0 here is rejected with an opaque error.
            let number = (i + 1) as i32;

            let body = ByteStream::read_from()
                .path(path)
                .offset(offset)
                .length(Length::Exact(this))
                .build()
                .await
                .with_context(|| format!("reading part {number} of {}", path.display()))?;

            let out = self
                .client
                .upload_part()
                .bucket(&self.bucket)
                .key(key)
                .upload_id(upload_id)
                .part_number(number)
                .body(body)
                .send()
                .await
                .with_context(|| format!("upload_part {number}/{parts}"))?;

            completed.push(
                CompletedPart::builder()
                    .part_number(number)
                    .set_e_tag(out.e_tag().map(|s| s.to_string()))
                    .build(),
            );
            info!("  part {number}/{parts} ({this} bytes)");
        }
        Ok(completed)
    }

    async fn head(&self, key: &str) -> Result<StoredObject> {
        let out = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .with_context(|| {
                format!(
                    "head_object s3://{}/{key} — upload not readable back",
                    self.bucket
                )
            })?;
        Ok(StoredObject {
            bytes: out.content_length().unwrap_or_default() as u64,
            // The API omits the field entirely for STANDARD rather than naming it.
            storage_class: out
                .storage_class()
                .map(|c| c.as_str().to_string())
                .unwrap_or_else(|| "STANDARD".to_string()),
        })
    }
}

/// Parse a storage-class name, rejecting typos rather than letting S3 default silently.
pub fn parse_storage_class(s: &str) -> Result<StorageClass> {
    // Compare against the known set rather than matching on `Unknown`, which the SDK
    // deprecates: new classes appear over time and matching would break on upgrade.
    if !StorageClass::values().contains(&s) {
        bail!(
            "unknown storage class '{s}'; expected one of: {}",
            StorageClass::values().join(", ")
        );
    }
    Ok(StorageClass::from(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_storage_classes_parse() {
        for name in ["STANDARD_IA", "DEEP_ARCHIVE", "STANDARD", "GLACIER_IR"] {
            let c = parse_storage_class(name).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(c.as_str(), name);
        }
    }

    #[test]
    fn typos_are_rejected_not_silently_defaulted() {
        // The failure this guards against: DEEP_ARCHIVE misspelled, everything appears to
        // work, and the bill arrives having stored terabytes at STANDARD prices.
        assert!(parse_storage_class("DEEP-ARCHIVE").is_err());
        assert!(parse_storage_class("deep_archive").is_err());
        assert!(parse_storage_class("").is_err());
    }

    #[test]
    fn part_maths_covers_the_file_exactly() {
        for len in [
            1u64,
            PART_SIZE - 1,
            PART_SIZE,
            PART_SIZE + 1,
            10 * PART_SIZE + 7,
        ] {
            let parts = len.div_ceil(PART_SIZE);
            let total: u64 = (0..parts).map(|i| PART_SIZE.min(len - i * PART_SIZE)).sum();
            assert_eq!(
                total, len,
                "parts must sum to the file length for len={len}"
            );
        }
    }
}
