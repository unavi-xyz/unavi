pub use semver::Version;

#[tarpc::service]
pub trait ControlService {
    async fn version() -> Version;
}
