use jose_jwk::{EcCurves, Jwk, Key};
use ring::signature::{ECDSA_P256_SHA256_ASN1, ECDSA_P384_SHA384_ASN1, VerificationAlgorithm};

pub fn verify_jwk_signature(jwk: &Jwk, signature: &[u8], signed_bytes: &[u8]) -> bool {
    match &jwk.key {
        Key::Ec(ec) => {
            let mut public_key = vec![0x04];
            public_key.extend(ec.x.as_ref());
            public_key.extend(ec.y.as_ref());

            let verified = match &ec.crv {
                EcCurves::P256 => ECDSA_P256_SHA256_ASN1.verify(
                    public_key.as_slice().into(),
                    signed_bytes.into(),
                    signature.into(),
                ),
                EcCurves::P384 => ECDSA_P384_SHA384_ASN1.verify(
                    public_key.as_slice().into(),
                    signed_bytes.into(),
                    signature.into(),
                ),
                _ => return false,
            };

            if verified.is_err() {
                return false;
            }
        }
        _ => return false,
    }

    true
}
