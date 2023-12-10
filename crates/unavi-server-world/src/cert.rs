use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyUsagePurpose};
use time::{Duration, OffsetDateTime};

/// Generate a self-signed certificate.
pub fn generate_certificate() -> wtransport::tls::Certificate {
    let ca = new_ca();

    let cert = ca.serialize_der().unwrap();
    let key = ca.serialize_private_key_der();

    wtransport::tls::Certificate::new(vec![cert], key)
}

fn new_ca() -> rcgen::Certificate {
    let mut params = CertificateParams::new(Vec::default());

    let (yesterday, tomorrow) = validity_period();

    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);
    params.not_before = yesterday;
    params.not_after = tomorrow;

    rcgen::Certificate::from_params(params).unwrap()
}

fn validity_period() -> (OffsetDateTime, OffsetDateTime) {
    let day = Duration::new(86400, 0);
    let yesterday = OffsetDateTime::now_utc().checked_sub(day).unwrap();
    let tomorrow = OffsetDateTime::now_utc().checked_add(day).unwrap();
    (yesterday, tomorrow)
}
