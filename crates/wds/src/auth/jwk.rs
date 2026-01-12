use jose_jwk::{EcCurves, Jwk, Key};
use signature::Verifier;

pub fn verify_jwk_signature(jwk: &Jwk, signature: &[u8], signed_bytes: &[u8]) -> bool {
    match &jwk.key {
        Key::Ec(ec) => {
            let mut pk_bytes = vec![0x04];
            pk_bytes.extend(ec.x.as_ref());
            pk_bytes.extend(ec.y.as_ref());

            match &ec.crv {
                EcCurves::P256 => {
                    let Ok(vk) = p256::ecdsa::VerifyingKey::from_sec1_bytes(&pk_bytes) else {
                        return false;
                    };
                    let Ok(sig) = p256::ecdsa::Signature::from_der(signature) else {
                        return false;
                    };
                    vk.verify(signed_bytes, &sig).is_ok()
                }
                EcCurves::P384 => {
                    let Ok(vk) = p384::ecdsa::VerifyingKey::from_sec1_bytes(&pk_bytes) else {
                        return false;
                    };
                    let Ok(sig) = p384::ecdsa::Signature::from_der(signature) else {
                        return false;
                    };
                    vk.verify(signed_bytes, &sig).is_ok()
                }
                _ => false,
            }
        }
        _ => false,
    }
}
