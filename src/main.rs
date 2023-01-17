use anyhow::Result;
use channel_runner::backends::{Backend, Docker, LXC};

#[tokio::main]
async fn main() -> Result<()> {
    let lxc = LXC::new("doiominicgay").await?;
    lxc.create(Some("alpine"), Some("edge")).await.unwrap();

    // let docker = Docker::new("alfredo-runner").await?;
    // _ = docker.create(None, None).await.unwrap();

    Ok(())
}
