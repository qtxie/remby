use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

const NONCE_LEN: usize = 12;

fn machine_key() -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let hostname = hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_default();
    let username = std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_default();
    let machine_id = format!("{}:{}", hostname, username);

    let mut key = [0u8; 32];
    let mut hasher = DefaultHasher::new();
    machine_id.hash(&mut hasher);
    let seed = hasher.finish();

    for i in 0..4 {
        let mut h = DefaultHasher::new();
        (seed, i).hash(&mut h);
        let bytes = h.finish().to_le_bytes();
        key[i * 8..(i + 1) * 8].copy_from_slice(&bytes);
    }
    key
}

pub fn encrypt(plaintext: &str) -> String {
    let key = machine_key();
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();

    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    base64_encode(&combined)
}

pub fn decrypt(encoded: &str) -> Option<String> {
    let combined = base64_decode(encoded)?;
    if combined.len() < NONCE_LEN {
        return None;
    }
    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);
    let key = machine_key();
    let cipher = Aes256Gcm::new_from_slice(&key).ok()?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let mut table = [0xFFu8; 256];
    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for (i, &c) in chars.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    let clean: Vec<u8> = s.bytes().filter(|&b| table[b as usize] != 0xFF || b == b'=').collect();
    let mut result = Vec::new();
    for chunk in clean.chunks(4) {
        if chunk.len() < 4 { break; }
        let a = table[chunk[0] as usize] as u32;
        let b = table[chunk[1] as usize] as u32;
        let c = if chunk[2] == b'=' { 0 } else { table[chunk[2] as usize] as u32 };
        let d = if chunk[3] == b'=' { 0 } else { table[chunk[3] as usize] as u32 };
        if a == 0xFF || b == 0xFF { return None; }
        result.push(((a << 2) | (b >> 4)) as u8);
        if chunk[2] != b'=' {
            result.push((((b & 0xF) << 4) | (c >> 2)) as u8);
        }
        if chunk[3] != b'=' {
            result.push((((c & 3) << 6) | d) as u8);
        }
    }
    Some(result)
}
