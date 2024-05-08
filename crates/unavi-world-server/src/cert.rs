use wtransport::{
    tls::{Certificate, CertificateChain, PrivateKey},
    Identity,
};

pub fn generate_tls_identity() -> Identity {
    let key = rcgen::generate_simple_self_signed(Vec::new()).unwrap();

    let cert = key.cert.der().to_vec();
    let key = key.key_pair.serialize_der();

    Identity::new(
        CertificateChain::new(vec![
            Certificate::from_der(cert).expect("invalid certificate")
        ]),
        PrivateKey::from_der_pkcs8(key),
    )
}
