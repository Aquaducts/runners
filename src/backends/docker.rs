use super::prelude::*;
use crate::docker::prelude::DockerInner;
use bollard::{
    container::{Config, CreateContainerOptions, StartContainerOptions},
    exec::StartExecOptions,
    models::ContainerCreateResponse,
};

pub struct Docker<'a> {
    pub name: &'a str,
    pub _inner: DockerInner,
}

#[async_trait]
impl<'a> Backend<'a> for Docker<'a> {
    type CreateResult = Result<ContainerCreateResponse>;
    async fn new_async(name: &'a str) -> Result<Self> {
        println!("0");
        let docker = DockerInner::new().await?;
        Ok(Self {
            _inner: docker,
            name,
        })
    }

    async fn create_async(&self, image: &'a str, release: &'a str) -> Self::CreateResult {
        println!("1");
        let container_opts = CreateContainerOptions { name: self.name };
        let container_image = format!("{image}:{release}");
        let container_cfg = Config::<&str> {
            image: Some(&container_image),
            attach_stdout: Some(true),
            cmd: Some(vec!["/usr/bin/git", "--help"]),
            tty: Some(true),
            working_dir: Some("/usr/"),
            ..Default::default()
        };
        println!("2");
        let created_container = self
            ._inner
            .0
            .create_container(Some(container_opts), container_cfg)
            .await?;

        Ok(created_container)
    }
}
