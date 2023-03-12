use super::prelude::*;
use anyhow::{bail, Result};
use bollard::{image::CreateImageOptions, service::CreateImageInfo};
use futures_util::StreamExt;
use std::{fs::File, io::Write, ops::Deref, process::ExitStatus};
use tar::Builder;
use tokio::process::Command;

pub struct Image<'a> {
    pub name: &'a str,
    pub tag: &'a str,
    pub docker: &'a DockerInner,
    pub create_image_info: Option<CreateImageInfo>,
}

impl<'a> Image<'a> {
    pub async fn create_image(
        git_url: &'a str,
        name: &'a str,
        tag: &'a str,
        docker: &'a DockerInner,
    ) -> Result<Image<'a>> {
        let tar_buffer = Vec::new();
        let mut tar_builder = Builder::new(tar_buffer);

        // TODO: use some like sqlite db to store the name of images so new ones dont have to be constantly made
        // TODO: this only works on linux for now because I need tmp directory :)
        let temp_dir = std::env::temp_dir().display().to_string();
        let temp_image_dir = &format!("{temp_dir}/channel-runner/images/{}/{}", name, tag);
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
            format!("channel-runner/images/{}/{}", name, tag),
            &temp_image_dir,
        )?;
        tar_builder.finish()?;
        let tar_buffer = tar_builder.get_ref();
        let options = Some(CreateImageOptions::<&str> {
            tag,
            repo: name,
            from_src: "-",
            ..Default::default()
        });
        let Some(Ok(image)) = docker.0.create_image::<&str>(options, Some(hyper::body::Bytes::from(tar_buffer.into_iter().map(|e| *e).collect::<Vec<u8>>()).into()), None).next().await else {
            bail!("Image was None?");
        };
        Ok(Self {
            name,
            tag,
            docker: &docker,
            create_image_info: Some(image),
        })
    }
}
