use p256::pkcs8::EncodePrivateKey;
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;

fn identity(key: P256KeyPair) -> Identity {
    let did = key.public().to_did();
    Identity::new(did, key)
}

#[test]
fn every_device_holding_the_key_derives_one_profile() {
    let key = P256KeyPair::generate();
    let pem = key.to_pkcs8_pem().expect("encode");
    let copy = P256KeyPair::from_pkcs8_pem(&pem).expect("decode");

    let here = ProfileSecret::derive(&identity(key)).expect("derive");
    let there = ProfileSecret::derive(&identity(copy)).expect("derive");
    assert_eq!(here.id(), there.id());
}

#[test]
fn another_key_derives_another_profile() {
    let one = ProfileSecret::derive(&identity(P256KeyPair::generate())).expect("derive");
    let two = ProfileSecret::derive(&identity(P256KeyPair::generate())).expect("derive");
    assert_ne!(one.id(), two.id());
}

#[test]
fn a_known_key_derives_a_known_profile() {
    let scalar = p256::SecretKey::from_slice(&[7; 32]).expect("scalar");
    let pem = scalar
        .to_pkcs8_pem(p256::pkcs8::LineEnding::CRLF)
        .expect("encode");
    let key = P256KeyPair::from_pkcs8_pem(&pem).expect("decode");

    let id = ProfileSecret::derive(&identity(key)).expect("derive").id();
    assert_eq!(
        id.to_string(),
        "be7bb6b75f05ec255e1cca61d30f037fe80a9d3686885eb72315d5183d189a03",
        "a changed derivation moves every user to a new profile"
    );
}
