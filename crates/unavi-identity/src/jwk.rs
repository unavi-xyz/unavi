use jose_jwk::{
    EcCurves,
    Jwk,
    Key,
};
use signature::Verifier;

#[derive(Debug, thiserror::Error)]
pub enum JwkError {
    #[error("unsupported key type")]
    UnsupportedKey,
    #[error("unsupported curve")]
    UnsupportedCurve,
    #[error("malformed public key")]
    MalformedKey,
    #[error("malformed signature")]
    MalformedSignature,
    #[error("signature does not verify")]
    Mismatch,
}

pub fn verify(jwk: &Jwk, signature: &[u8], signed_bytes: &[u8]) -> Result<(), JwkError> {
    let Key::Ec(ec) = &jwk.key else {
        return Err(JwkError::UnsupportedKey);
    };

    match ec.crv {
        EcCurves::P256 => {
            let x = coordinate::<32>(&ec.x)?;
            let y = coordinate::<32>(&ec.y)?;

            let point = p256::EncodedPoint::from_affine_coordinates(&x.into(), &y.into(), false);
            let key = p256::ecdsa::VerifyingKey::from_encoded_point(&point)
                .map_err(|_| JwkError::MalformedKey)?;
            let signature = p256::ecdsa::Signature::from_der(signature)
                .map_err(|_| JwkError::MalformedSignature)?;

            key.verify(signed_bytes, &signature)
                .map_err(|_| JwkError::Mismatch)
        }
        EcCurves::P384 => {
            let x = coordinate::<48>(&ec.x)?;
            let y = coordinate::<48>(&ec.y)?;

            let point = p384::EncodedPoint::from_affine_coordinates(&x.into(), &y.into(), false);
            let key = p384::ecdsa::VerifyingKey::from_encoded_point(&point)
                .map_err(|_| JwkError::MalformedKey)?;
            let signature = p384::ecdsa::Signature::from_der(signature)
                .map_err(|_| JwkError::MalformedSignature)?;

            key.verify(signed_bytes, &signature)
                .map_err(|_| JwkError::Mismatch)
        }
        _ => Err(JwkError::UnsupportedCurve),
    }
}

/// A JWK coordinate is unpadded once decoded, so one short by a leading zero
/// byte is well-formed yet the wrong width for the curve.
fn coordinate<const N: usize>(value: &[u8]) -> Result<[u8; N], JwkError> {
    <[u8; N]>::try_from(value).map_err(|_| JwkError::MalformedKey)
}
