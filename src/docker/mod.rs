pub mod images;

pub mod prelude {
    use anyhow::{bail, Result};
    use bollard::Docker;
    pub struct DockerInner(pub Docker);
    impl DockerInner {
        pub async fn new() -> Result<Self> {
            let docker = Docker::connect_with_socket_defaults()?;
            if docker.ping().await.is_err() {
                bail!("Failed to ping docker.");
            }
            Ok(Self(docker))
        }
    }
}
