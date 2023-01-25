use super::prelude::*;
use anyhow::{bail, Result};
use bollard::{image::CreateImageOptions, service::CreateImageInfo};
use futures_util::StreamExt;
use std::{fs::File, io::Write, process::ExitStatus};
use tar::Builder;
use tokio::process::Command;

pub struct Image<'a> {
    pub name: &'a str,
    pub tag: &'a str,
    pub docker: &'a DockerInner,
    pub create_image_info: Option<CreateImageInfo>,
}

impl<'a> Image<'a> {
    pub async fn create_image_info(
        name: &'a str,
        tag: &'a str,
        docker: &'a DockerInner,
    ) -> Result<Image<'a>> {
        let options = Some(CreateImageOptions::<&str> {
            tag,
            repo: name,
            from_src: "-",
            ..Default::default()
        });
        let Some(Ok(image)) = docker.0.create_image::<&str>(options, None, None).next().await else {
            bail!("Image was None?");
        };
        Ok(Self {
            name,
            tag,
            docker: &docker,
            create_image_info: Some(image),
        })
    }

    pub async fn create_image(&self, git_url: &'a str) -> Result<()> {
        let tar_buffer = File::create("temp-archive.tar")?; // Vec::new();
        let mut tar_builder = Builder::new(tar_buffer);

        // TODO: use some like sqlite db to store the name of images so new ones dont have to be constantly made
        // TODO: this only works on linux for now because I need tmp directory :)
        let temp_dir = std::env::temp_dir().display().to_string();
        let temp_image_dir = &format!(
            "{temp_dir}/channel-runner/images/{}/{}",
            self.name, self.tag
        );
        let Ok(command) = Command::new("git")
            .args(&["clone", git_url, &temp_image_dir, "--recursive"])
            .spawn()?
            .wait()
            .await else {
                bail!("Cloning git repo to a temporary directory failed.");
            };
        if !command.success() {
            bail!("Git command was not successful.");
        }

        tar_builder.append_dir_all(
            format!("channel-runner/images/{}/{}", self.name, self.tag),
            &temp_image_dir,
        )?;
        tar_builder.finish()?;

        Ok(())
    }
}
