use std::io;
use std::path::{Path, PathBuf};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use sha2::{Digest, Sha256};

const APP_SECRET: &[u8] = b"bongo-penguin-cosmic.v1.dont-cheat.4e11d28a-5f7b-43c9";
const NONCE_LEN: usize = 12;
const COUNT_LEN: usize = 8;
const TAG_LEN: usize = 16;

pub fn load() -> Option<u64> {
    let path = data_path()?;
    let key = derive_key()?;
    load_at(&path, &key)
}

pub fn save(count: u64) -> io::Result<()> {
    let path = data_path().ok_or_else(|| io::Error::other("no data path"))?;
    let key = derive_key().ok_or_else(|| io::Error::other("no key material"))?;
    save_at(&path, count, &key)
}

fn load_at(path: &Path, key: &[u8; 32]) -> Option<u64> {
    let data = std::fs::read(path).ok()?;
    decrypt(key, &data)
}

fn save_at(path: &Path, count: u64, key: &[u8; 32]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let out = encrypt(key, count).map_err(|_| io::Error::other("encrypt failed"))?;
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, &out)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn encrypt(key: &[u8; 32], count: u64) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher.encrypt(nonce, &count.to_le_bytes()[..])?;
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

fn decrypt(key: &[u8; 32], data: &[u8]) -> Option<u64> {
    if data.len() != NONCE_LEN + COUNT_LEN + TAG_LEN {
        return None;
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&data[..NONCE_LEN]);
    let pt = cipher.decrypt(nonce, &data[NONCE_LEN..]).ok()?;
    let buf: [u8; COUNT_LEN] = pt.try_into().ok()?;
    Some(u64::from_le_bytes(buf))
}

fn data_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("bongo-penguin-cosmic").join("count.dat"))
}

fn derive_key() -> Option<[u8; 32]> {
    let machine_id = std::fs::read_to_string("/etc/machine-id").ok()?;
    let mut h = Sha256::new();
    h.update(APP_SECRET);
    h.update(machine_id.trim().as_bytes());
    Some(h.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        for (i, v) in k.iter_mut().enumerate() {
            *v = (i as u8).wrapping_mul(7).wrapping_add(3);
        }
        k
    }

    #[test]
    fn encrypt_then_decrypt_round_trip() {
        let key = test_key();
        let data = encrypt(&key, 12_345).unwrap();
        assert_eq!(decrypt(&key, &data), Some(12_345));
    }

    #[test]
    fn encrypt_zero_round_trips() {
        let key = test_key();
        let data = encrypt(&key, 0).unwrap();
        assert_eq!(decrypt(&key, &data), Some(0));
    }

    #[test]
    fn encrypt_max_u64_round_trips() {
        let key = test_key();
        let data = encrypt(&key, u64::MAX).unwrap();
        assert_eq!(decrypt(&key, &data), Some(u64::MAX));
    }

    #[test]
    fn encrypted_payload_has_expected_length() {
        let key = test_key();
        let data = encrypt(&key, 42).unwrap();
        assert_eq!(data.len(), NONCE_LEN + COUNT_LEN + TAG_LEN);
    }

    #[test]
    fn encrypt_uses_fresh_nonce_each_call() {
        let key = test_key();
        let a = encrypt(&key, 42).unwrap();
        let b = encrypt(&key, 42).unwrap();
        assert_ne!(a, b, "two encryptions must differ thanks to a fresh nonce");
    }

    #[test]
    fn decrypt_rejects_empty_and_short_data() {
        let key = test_key();
        assert_eq!(decrypt(&key, &[]), None);
        assert_eq!(decrypt(&key, &[0u8; 10]), None);
        assert_eq!(
            decrypt(&key, &[0u8; NONCE_LEN + COUNT_LEN + TAG_LEN - 1]),
            None
        );
    }

    #[test]
    fn decrypt_rejects_oversized_data() {
        let key = test_key();
        let data = vec![0u8; NONCE_LEN + COUNT_LEN + TAG_LEN + 1];
        assert_eq!(decrypt(&key, &data), None);
    }

    #[test]
    fn decrypt_rejects_tampered_auth_tag() {
        let key = test_key();
        let mut data = encrypt(&key, 1_234).unwrap();
        let last = data.len() - 1;
        data[last] ^= 0x01;
        assert_eq!(decrypt(&key, &data), None);
    }

    #[test]
    fn decrypt_rejects_tampered_nonce() {
        let key = test_key();
        let mut data = encrypt(&key, 1_234).unwrap();
        data[0] ^= 0xFF;
        assert_eq!(decrypt(&key, &data), None);
    }

    #[test]
    fn decrypt_rejects_tampered_ciphertext_body() {
        let key = test_key();
        let mut data = encrypt(&key, 9_999).unwrap();
        data[NONCE_LEN] ^= 0xFF;
        assert_eq!(decrypt(&key, &data), None);
    }

    #[test]
    fn decrypt_rejects_wrong_key() {
        let key_a = test_key();
        let mut key_b = test_key();
        key_b[0] ^= 0xAA;
        let data = encrypt(&key_a, 7).unwrap();
        assert_eq!(decrypt(&key_b, &data), None);
    }

    #[test]
    fn save_then_load_round_trips_via_filesystem() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("count.dat");
        let key = test_key();
        save_at(&path, 98_765, &key).unwrap();
        assert!(path.exists());
        assert_eq!(load_at(&path, &key), Some(98_765));
    }

    #[test]
    fn save_creates_parent_directories() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("deeper").join("count.dat");
        save_at(&path, 1, &test_key()).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn save_overwrites_previous_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("count.dat");
        let key = test_key();
        save_at(&path, 10, &key).unwrap();
        save_at(&path, 20, &key).unwrap();
        assert_eq!(load_at(&path, &key), Some(20));
    }

    #[test]
    fn load_returns_none_for_missing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("absent.dat");
        assert_eq!(load_at(&path, &test_key()), None);
    }

    #[test]
    fn load_returns_none_for_garbage_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("count.dat");
        std::fs::write(&path, b"not a real count file").unwrap();
        assert_eq!(load_at(&path, &test_key()), None);
    }

    #[test]
    fn load_returns_none_when_file_written_with_a_different_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("count.dat");
        let key_a = test_key();
        let mut key_b = test_key();
        key_b[5] ^= 0xF0;
        save_at(&path, 123, &key_a).unwrap();
        assert_eq!(load_at(&path, &key_b), None);
    }

    #[test]
    fn save_leaves_no_tmp_file_after_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("count.dat");
        save_at(&path, 1, &test_key()).unwrap();
        for entry in std::fs::read_dir(dir.path()).unwrap().flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            assert!(!name.starts_with("count.tmp."), "stray tmp file: {name}");
        }
    }

    #[test]
    fn derive_key_is_deterministic_when_machine_id_exists() {
        // This test is informational: on hosts without /etc/machine-id (some
        // containers), derive_key returns None. We only assert determinism when
        // it is available.
        if let (Some(a), Some(b)) = (derive_key(), derive_key()) {
            assert_eq!(a, b, "derive_key must be deterministic");
        }
    }

    #[test]
    fn data_path_uses_xdg_data_home_when_set() {
        // Run in a subprocess-safe way: we don't mutate the process env here.
        // Instead we reimplement the contract: given XDG_DATA_HOME, the path is
        // <XDG_DATA_HOME>/bongo-penguin-cosmic/count.dat. We don't touch env to
        // stay safe under parallel tests — just assert the shape via the real
        // fn when XDG_DATA_HOME is already set.
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
            let expected = PathBuf::from(xdg)
                .join("bongo-penguin-cosmic")
                .join("count.dat");
            assert_eq!(data_path(), Some(expected));
        } else if let Some(home) = std::env::var_os("HOME") {
            let expected = PathBuf::from(home)
                .join(".local/share")
                .join("bongo-penguin-cosmic")
                .join("count.dat");
            assert_eq!(data_path(), Some(expected));
        }
    }
}
