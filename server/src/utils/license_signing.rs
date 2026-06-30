use crate::core::{config::LicenseSigningConfig, my_error::AppError};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const LICENSE_ALGORITHM: &str = "Ed25519";

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct LicensePayload {
    pub version: i32,
    pub app_id: String,
    pub app_key_hash: String,
    pub device_id: String,
    pub license_type: String,
    pub expire_time: Option<i64>,
    pub remain_count: Option<i32>,
    pub issued_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct SignedLicense {
    pub payload: LicensePayload,
    pub signature: String,
    pub key_id: String,
    pub algorithm: String,
}

pub fn sign_license(
    config: &LicenseSigningConfig,
    private_key_b64: &str,
    payload: LicensePayload,
) -> Result<SignedLicense, AppError> {
    let signing_key = signing_key_from_private_key_b64(private_key_b64)?;
    let payload_bytes = canonical_payload_bytes(&payload)?;
    let signature = signing_key.sign(&payload_bytes);
    Ok(SignedLicense {
        payload,
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        key_id: config.key_id.clone(),
        algorithm: LICENSE_ALGORITHM.to_string(),
    })
}

pub fn canonical_payload_bytes(payload: &LicensePayload) -> Result<Vec<u8>, AppError> {
    serde_json::to_vec(payload).map_err(AppError::from)
}

pub fn app_key_hash(app_key: &str) -> String {
    let digest = Sha256::digest(app_key.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn generate_private_key_b64() -> Result<String, AppError> {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).map_err(|error| {
        AppError::Message(format!("generate license signing key failed: {error}"))
    })?;
    Ok(URL_SAFE_NO_PAD.encode(seed))
}

pub fn public_key_b64_from_private_key(private_key_b64: &str) -> Result<String, AppError> {
    let signing_key = signing_key_from_private_key_b64(private_key_b64)?;
    Ok(public_key_b64_from_verifying_key(
        &signing_key.verifying_key(),
    ))
}

pub fn public_key_b64_from_verifying_key(verifying_key: &VerifyingKey) -> String {
    URL_SAFE_NO_PAD.encode(verifying_key.to_bytes())
}

fn signing_key_from_private_key_b64(private_key_b64: &str) -> Result<SigningKey, AppError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(private_key_b64.trim())
        .map_err(|_| AppError::Message("invalid license signing private key".to_string()))?;
    let key_bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        AppError::Message("license signing private key must decode to 32 bytes".to_string())
    })?;
    Ok(SigningKey::from_bytes(&key_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signs_license_payload() {
        let config = LicenseSigningConfig {
            key_id: "license-v1".into(),
        };
        let private_key_b64 = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY";
        let payload = LicensePayload {
            version: 1,
            app_id: "com.example.app".into(),
            app_key_hash: app_key_hash("app-key"),
            device_id: "device-1".into(),
            license_type: "time".into(),
            expire_time: Some(1_893_456_000),
            remain_count: None,
            issued_at: 1_700_000_000,
        };

        let license = sign_license(&config, private_key_b64, payload).expect("signed license");

        assert_eq!(license.algorithm, LICENSE_ALGORITHM);
        assert_eq!(license.key_id, "license-v1");
        assert!(license.signature.len() > 32);
    }

    #[test]
    fn derives_public_key_from_private_seed() {
        let public_key =
            public_key_b64_from_private_key("MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY")
                .expect("public key");

        assert_eq!(public_key, "I7xUkSwebpLEqGglyGfif_3FVb_71CRPF6Jqv__ull0");
    }
}
