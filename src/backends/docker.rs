use super::prelude::*;
use bollard::{
    container::{Config, CreateContainerOptions},
    models::ContainerCreateResponse,
    Docker as DockerInner,
};

pub struct Docker<'a> {
    pub name: &'a str,
    pub _inner: DockerInner,
}

#[async_trait]
impl<'a> Backend<'a> for Docker<'a> {
    type CreateResult = Result<ContainerCreateResponse>;
    async fn new(name: &'a str) -> Result<Self> {
        let docker = DockerInner::connect_with_socket_defaults()?;
        if docker.ping().await.is_err() {
            bail!("Failed to ping docker.");
        }
        Ok(Self {
            _inner: docker,
            name,
        })
    }

    async fn create(&self, _: Option<&'a str>, _: Option<&'a str>) -> Self::CreateResult {
        let container_opts = CreateContainerOptions { name: self.name };
        let container_cfg = Config::<String> {
            attach_stdout: Some(true),
            cmd: Some(vec!["echo".to_string(), "\"hi\"".to_string()]),
            ..Default::default()
        };

        Ok(self
            ._inner
            .create_container(Some(container_opts), container_cfg)
            .await?)
    }
}
