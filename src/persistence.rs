use std::io;
use std::path::PathBuf;

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
    let data = std::fs::read(&path).ok()?;
    if data.len() != NONCE_LEN + COUNT_LEN + TAG_LEN {
        return None;
    }
    let key = derive_key()?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(&data[..NONCE_LEN]);
    let pt = cipher.decrypt(nonce, &data[NONCE_LEN..]).ok()?;
    let buf: [u8; COUNT_LEN] = pt.try_into().ok()?;
    Some(u64::from_le_bytes(buf))
}

pub fn save(count: u64) -> io::Result<()> {
    let path = data_path().ok_or_else(|| io::Error::other("no data path"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let key = derive_key().ok_or_else(|| io::Error::other("no key material"))?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ct = cipher
        .encrypt(nonce, &count.to_le_bytes()[..])
        .map_err(|_| io::Error::other("encrypt failed"))?;

    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);

    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, &out)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
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
