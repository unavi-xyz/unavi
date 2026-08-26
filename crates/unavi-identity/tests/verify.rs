use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::{
    resolve::Resolver,
    signed_bytes::{
        Signable,
        VerifyError,
    },
};
use xdid::methods::key::keys::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

#[derive(Serialize, Deserialize)]
struct Claim(u32);

impl Signable for Claim {
    const SIGNING_CONTEXT: &'static str = "test/claim";
}

#[tokio::test]
async fn a_signature_verifies_against_its_signers_did() {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    let signed = Claim(7).sign(&key).expect("sign");

    signed
        .verify(&did, &Resolver::new().expect("resolver"))
        .await
        .expect("a DID's own authentication key must verify its signature");
}

#[tokio::test]
async fn a_signature_does_not_verify_against_another_did() {
    let key = P256KeyPair::generate();
    let other = P256KeyPair::generate().public().to_did();
    let signed = Claim(7).sign(&key).expect("sign");

    assert!(
        matches!(
            signed
                .verify(&other, &Resolver::new().expect("resolver"))
                .await,
            Err(VerifyError::NotSigned(_))
        ),
        "holding one key must not let a signer speak for another DID"
    );
}
