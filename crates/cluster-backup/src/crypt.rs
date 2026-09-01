//! Public-key encryption of a backup archive.
//!
//! The machine producing these archives is the machine being backed up, and it runs
//! unattended at 02:30. A passphrase would have to live in a file on that host — so
//! anyone who stole the archive would also hold its key. Instead the host carries only an
//! age *recipient* (a public key): it can encrypt and upload, and cannot read back a single
//! byte of what it sent. The identity file never touches it.

use anyhow::{Context, Result, anyhow};
use std::io::Write;
use std::str::FromStr;

/// An age recipient the archive is encrypted to.
#[derive(Debug, Clone)]
pub struct Recipient(age::x25519::Recipient);

impl Recipient {
    /// Parse an `age1…` public key.
    ///
    /// Rejected early and loudly on purpose: the alternative is discovering the key is
    /// malformed after several gigabytes have already been compressed.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        age::x25519::Recipient::from_str(s)
            .map(Recipient)
            .map_err(|e| anyhow!("not a valid age recipient ({e}); expected an 'age1…' public key"))
    }
}

/// Wrap `out` so everything written to it is encrypted to `recipient`.
///
/// The returned writer **must** be finished with [`finish`] — dropping it silently
/// truncates the age payload and produces a file that looks fine until you try to decrypt it.
pub fn wrap<W: Write>(recipient: &Recipient, out: W) -> Result<age::stream::StreamWriter<W>> {
    let encryptor =
        age::Encryptor::with_recipients(std::iter::once(&recipient.0 as &dyn age::Recipient))
            .context("building age encryptor")?;
    encryptor.wrap_output(out).context("starting age stream")
}

/// Finalise an age stream, flushing its last chunk.
pub fn finish<W: Write>(w: age::stream::StreamWriter<W>) -> Result<W> {
    w.finish().context("finalising age stream")
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::ExposeSecret;
    use std::io::Read;

    #[test]
    fn rejects_a_bad_recipient() {
        assert!(Recipient::parse("not-a-key").is_err());
        assert!(Recipient::parse("").is_err());
    }

    #[test]
    fn round_trips_through_a_real_identity() {
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();
        let recipient = Recipient::parse(&pubkey).expect("parse own public key");

        let plaintext = b"SEALED-SECRETS-MASTER-KEY";
        let mut ciphertext = Vec::new();
        {
            let mut w = wrap(&recipient, &mut ciphertext).expect("wrap");
            w.write_all(plaintext).expect("write");
            finish(w).expect("finish");
        }
        assert_ne!(
            &ciphertext[..],
            &plaintext[..],
            "output must not be plaintext"
        );

        let decryptor = age::Decryptor::new(&ciphertext[..]).expect("decryptor");
        let mut r = decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .expect("decrypt");
        let mut got = Vec::new();
        r.read_to_end(&mut got).expect("read");
        assert_eq!(got, plaintext);
    }

    #[test]
    fn a_different_identity_cannot_decrypt() {
        let mine = age::x25519::Identity::generate();
        let theirs = age::x25519::Identity::generate();
        let recipient = Recipient::parse(&mine.to_public().to_string()).expect("parse");

        let mut ciphertext = Vec::new();
        {
            let mut w = wrap(&recipient, &mut ciphertext).expect("wrap");
            w.write_all(b"secret").expect("write");
            finish(w).expect("finish");
        }

        let decryptor = age::Decryptor::new(&ciphertext[..]).expect("decryptor");
        assert!(
            decryptor
                .decrypt(std::iter::once(&theirs as &dyn age::Identity))
                .is_err(),
            "an unrelated identity must not decrypt the archive"
        );
        // Keep the import meaningful: identities are secret material.
        let _ = mine.to_string().expose_secret().len();
    }
}
