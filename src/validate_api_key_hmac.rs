use hmac::{Hmac, Mac, digest::KeyInit};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;
pub fn validate_api_key_hmac(api_key: &str, secret: &str) -> bool {
    let mut parts = api_key.split(".");
    let unique_identifier = match parts.next() {
        Some(id) => id,
        None => return false,
    };
    let client_signature = match parts.next() {
        Some(sig) => sig,
        None => return false,
    };
    if parts.next().is_some() {
        return false;
    };
    let client_signature_bytes = match hex::decode(client_signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let mut mac: HmacSha256 = match KeyInit::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => {
            eprintln!("Error creating HMAC from secret.");
            return false;
        }
    };
    mac.update(unique_identifier.as_bytes());
    let expected_signature = mac.finalize().into_bytes();
    client_signature_bytes
        .ct_eq(&expected_signature.as_slice())
        .into()
}
