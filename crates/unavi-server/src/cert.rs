use std::str::FromStr;

use p256::{
    SecretKey,
    ecdsa::{DerSignature, SigningKey},
    elliptic_curve::rand_core::OsRng,
    pkcs8::{EncodePrivateKey, EncodePublicKey},
};
use spki::der::Encode;
use time::Duration;
use x509_cert::{
    builder::{Builder, CertificateBuilder, Profile},
    name::Name,
    serial_number::SerialNumber,
    spki::SubjectPublicKeyInfoOwned,
    time::Validity,
};

use crate::DIRS;

pub struct CertRes {
    pub cert: Vec<u8>,
    pub private_key: Vec<u8>,
}

pub async fn get_or_generate_cert() -> anyhow::Result<CertRes> {
    let dir = DIRS.config_dir();

    let cert_path = {
        let mut dir = dir.to_path_buf();
        dir.push("cert.der");
        dir
    };

    let key_path = {
        let mut dir = dir.to_path_buf();
        dir.push("key.der");
        dir
    };

    if cert_path.exists() && key_path.exists() {
        let cert = tokio::fs::read(cert_path).await?;
        let private_key = tokio::fs::read(key_path).await?;
        Ok(CertRes { cert, private_key })
    } else {
        let secret_key = SecretKey::random(&mut OsRng);
        let pub_key = SubjectPublicKeyInfoOwned::try_from(
            secret_key.public_key().to_public_key_der()?.as_bytes(),
        )?;

        let signer = SigningKey::from(secret_key);
        let signer_der = signer.to_pkcs8_der()?;

        let serial_number = SerialNumber::from(1u32);
        let validity = Validity::from_now(Duration::days(365).try_into()?)?;
        let subject =
            Name::from_str("CN=World domination corporation,O=World domination Inc,C=US")?;

        let builder = CertificateBuilder::new(
            Profile::Root,
            serial_number,
            validity,
            subject,
            pub_key,
            &signer,
        )?;

        let certificate = builder.build::<DerSignature>()?;
        let cert_der = certificate.to_der()?;

        tokio::fs::write(cert_path, &cert_der).await?;
        signer_der.write_der_file(key_path)?;

        Ok(CertRes {
            cert: cert_der,
            private_key: signer_der.to_bytes().to_vec(),
        })
    }
}
