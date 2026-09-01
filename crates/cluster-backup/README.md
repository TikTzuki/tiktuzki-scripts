# cluster-backup

Archives, encrypts and uploads a microk8s backup *run* to S3.

A run is one timestamped directory produced by the collection step in `tiktuzki-gitops`
(`infra/backup/backup.sh`):

```
20260901-093329/
├── MANIFEST.txt              written last — the marker that collection finished
├── sealed-secrets-key.yaml
├── secrets/                  every Secret in every namespace
├── node/                     microk8s CA + certs, dqlite, netplan, NetBird, node-info
├── db/                       pg_dumpall output
└── data/                     volume tarballs
```

This crate replaces `infra/backup/upload-s3.sh`. Same split and cadence; it does the work in
one process instead of shelling out to `tar`, `gpg` and `aws`, and it uses **age** rather than
GPG, so there is no keyring to manage on the node.

## Two halves, two storage classes

Glacier Deep Archive takes **12–48 hours** to restore. If the Sealed-Secrets master key and
the microk8s CA live only there, a rebuild cannot *start* for up to two days. Those pieces are
tens of kilobytes, so keeping them instantly retrievable costs almost nothing.

| half | contents | default class | why |
|---|---|---|---|
| `critical` | manifest, master key, secrets, node state | `STANDARD_IA` | needed in the first five minutes of a rebuild |
| `bulk` | `db/`, `data/` | `DEEP_ARCHIVE` | useless until the cluster is already back up |

Keys are `s3://<bucket>/<prefix>/<half>/<stamp>.tar.zst.age`.

## Cadence

Deep Archive bills a **180-day minimum per object**, whether or not you delete it sooner.
Uploading gigabytes daily therefore bills roughly 180 copies at steady state, not 7. So the
critical half goes up every run and the bulk half only on `--bulk-dow` (default Sunday).

## Encryption

The host holds only an age **recipient** — a public key. It can encrypt and upload; it cannot
read back a byte of what it sent. The identity file never touches it.

This matters because the machine producing the archive *is* the machine being backed up. A
passphrase would have to live in a file on that host, so anyone who stole an archive would
also hold its key.

```bash
# On your LAPTOP, never on the node:
age-keygen -o backup-identity.txt        # prints the public key to stderr
# => Public key: age1qz…

# Store backup-identity.txt in your password manager. Losing it loses every archive.
```

The pipeline is `tar -> zstd -> age`, in that order. Compressing after encrypting would
inflate the archive, since age output is indistinguishable from random.

## Usage

```bash
# What would be uploaded?
cluster-backup plan --backup-dir /srv/k8s-volumes/backups

# Do it.
cluster-backup upload \
  --backup-dir /srv/k8s-volumes/backups \
  --bucket tiktuzki-cluster-backup \
  --prefix node1 \
  --region ap-southeast-1 \
  --recipient age1qz…

# Ship the bulk half regardless of the weekday.
cluster-backup upload … --force-bulk
```

Every flag has an environment variable (`S3_BUCKET`, `AGE_RECIPIENT`, `AWS_REGION`,
`CRITICAL_CLASS`, `BULK_CLASS`, `UPLOAD_BULK_DOW`, `FORCE_BULK`) so systemd can configure it
with `Environment=`.

## Safety properties

- **Refuses incomplete runs.** No `MANIFEST.txt` means collection died partway. Shipping that
  would create something that looks like a backup and is not one.
- **Validates before working.** Recipient, storage classes and weekday are checked before any
  compression, so a typo costs a second rather than an hour of CPU.
- **Rejects unknown storage classes.** `DEEP-ARCHIVE` is not `DEEP_ARCHIVE`; letting S3 quietly
  default to `STANDARD` is how you store terabytes at 20× the intended price.
- **Verifies after uploading.** A successful `PutObject` only means the request was accepted.
  A `head_object` afterwards confirms size, and warns if the stored class is not the one asked
  for (a bucket lifecycle rule can override it).
- **Multipart above 100 MiB**, 64 MiB parts, and it *aborts* the upload on failure — orphaned
  parts are billed until a lifecycle rule reaps them, and most buckets have no such rule.
- **Archives are mode 0600** from creation, and `fsync`ed before being reported as done.

## IAM

Give the node `s3:PutObject` and nothing else. A host that cannot delete or read its own
backups cannot be made to destroy them, which is the whole point of pushing them off the box.

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": ["s3:PutObject", "s3:AbortMultipartUpload"],
      "Resource": "arn:aws:s3:::tiktuzki-cluster-backup/node1/*"
    }
  ]
}
```

`head_object` verification needs `s3:GetObject` too. If you would rather not grant read access
to the node, drop it and accept that uploads are unverified — the trade is real, and the
least-privilege side is the better default for an unattended host.

Pair it with versioning and Object Lock on the bucket so even a stolen key cannot rewrite
history.

## Testing

```bash
cargo test -p cluster-backup
```

Covers the plan/manifest logic, an age round-trip (including that a *different* identity
cannot decrypt), a full `tar -> zstd -> age` archive round-trip back to the original bytes,
archive permissions, storage-class parsing, and multipart part arithmetic.

There is no test against real S3. The upload path has been exercised end to end against the
live API up to the credential check.
