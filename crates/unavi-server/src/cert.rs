use std::str::FromStr;

use p256::{
    SecretKey,
    ecdsa::{DerSignature, SigningKey},
    elliptic_curve::rand_core::OsRng,
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding},
};
use spki::der::{DecodePem, EncodePem};
use time::Duration;
use x509_cert::{
    Certificate,
    builder::{Builder, CertificateBuilder, Profile},
    name::Name,
    serial_number::SerialNumber,
    spki::SubjectPublicKeyInfoOwned,
    time::Validity,
};

use crate::DIRS;

pub struct CertRes {
    pub cert: Certificate,
    pub key: SecretKey,
}

pub async fn get_or_generate_cert() -> anyhow::Result<CertRes> {
    let dir = DIRS.config_dir();

    let cert_path = {
        let mut dir = dir.to_path_buf();
        dir.push("cert.pem");
        dir
    };

    let key_path = {
        let mut dir = dir.to_path_buf();
        dir.push("key.pem");
        dir
    };

    if cert_path.exists() && key_path.exists() {
        let cert_pem = tokio::fs::read(cert_path).await?;
        let key_pem = tokio::fs::read_to_string(key_path).await?;

        let cert = Certificate::from_pem(cert_pem)?;
        let key = SecretKey::from_pkcs8_pem(&key_pem)?;

        Ok(CertRes { cert, key })
    } else {
        let key = SecretKey::random(&mut OsRng);
        let pub_key =
            SubjectPublicKeyInfoOwned::try_from(key.public_key().to_public_key_der()?.as_bytes())?;

        let signer = SigningKey::from(key.clone());

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

        let cert_pem = certificate.to_pem(LineEnding::LF)?;
        let key_pem = signer.to_pkcs8_pem(LineEnding::LF)?;

        let cert = Certificate::from_pem(&cert_pem)?;

        tokio::fs::write(cert_path, &cert_pem).await?;
        tokio::fs::write(key_path, &key_pem).await?;

        Ok(CertRes { cert, key })
    }
}
