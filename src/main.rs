use std::any::TypeId;

use anyhow::Result;
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use channel_runner::{
    backends::{Backend, Docker, LXC},
    config::CONFIG,
    Websocket,
};

use common::{
    events::{CreateJobRun, Identify, RepoConfig, RequestRepoConfig},
    websocket::{OpCodes, WebsocketMessage},
};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    sudo::escalate_if_needed().unwrap();

    let docker = channel_runner::docker::prelude::DockerInner::new().await?;
    let image =
        channel_runner::docker::images::Image::create_image_info("test-imgae", "latset", &docker)
            .await?;

    image
        .create_image("https://github.com/pylon-sh/powerline.git")
        .await?;

    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::INFO)
    //     .pretty()
    //     .init();

    // sudo::escalate_if_needed().unwrap();
    // let websocket = Websocket::new().await?;
    // let (_, _, _) = websocket.start().await;

    // let mut reader = websocket.reader.lock().await;
    // while let Some(message) = reader.recv().await {
    //     match message.op {
    //         common::websocket::OpCodes::EventCreate => {
    //             if let Some(event) = message.event {
    //                 println!("{:?}", event);
    //                 println!("h7");
    //                 println!("hi1");
    //                 let d_any = event.as_any();
    //                 println!("{:?}", d_any.type_id());
    //                 println!("hi");
    //                 println!("hi3");
    //                 println!("hi6");
    //                 if d_any.type_id() == TypeId::of::<CreateJobRun>() {
    //                     let Some(data) = d_any.downcast_ref::<CreateJobRun>() else {
    //                         continue;
    //                     };

    //                     websocket.send(WebsocketMessage {
    //                         op: OpCodes::EventCreate,
    //                         event: Some(Box::new(RequestRepoConfig {
    //                             repo: data.job.repo,
    //                         })),
    //                     })?;

    //                     let Some(possibly_repo_config) = reader.recv().await else {
    //                         continue;
    //                     };
    //                     let Some(_event) = possibly_repo_config.event else {
    //                         continue;
    //                     };
    //                     let d_any = _event.as_any();

    //                     if d_any.type_id() != TypeId::of::<RepoConfig>() {
    //                         continue;
    //                     }

    //                     let repo = d_any.downcast_ref::<RepoConfig>().unwrap();
    //                     /*****************************************************************************
    //                     THE FOLLOWING CODE WILL BE MOVED IN THE FUTURE. DO NOT DEPEND ON IT BEING HERE
    //                     *****************************************************************************/
    //                     if &CONFIG.runner.backend.name.to_lowercase() == "docker" {
    //                         println!("4");
    //                         let container_name = format!("runner-{}-{}", "temp_value", data.job.id);
    //                         let docker = Docker::new_async(&container_name).await?;
    //                         println!("5");
    //                         let container = docker
    //                             .create_async(
    //                                 &CONFIG.runner.backend.image,
    //                                 &CONFIG.runner.backend.release,
    //                             )
    //                             .await?;
    //                         let container_id = container.id;
    //                         docker
    //                             ._inner
    //                             .0
    //                             .start_container::<String>(&container_id, None)
    //                             .await?;
    //                         println!("6");
    //                         let exec = docker
    //                             ._inner
    //                             .0
    //                             .create_exec(
    //                                 &container_id,
    //                                 CreateExecOptions::<&str> {
    //                                     working_dir: Some("/usr/pieces"),
    //                                     cmd: Some(vec!["cargo", "build"]),
    //                                     attach_stdout: Some(true),
    //                                     attach_stderr: Some(true),
    //                                     ..Default::default()
    //                                 },
    //                             )
    //                             .await?;
    //                         if let StartExecResults::Attached { mut output, .. } = docker
    //                             ._inner
    //                             .0
    //                             .start_exec(
    //                                 &exec.id,
    //                                 Some(StartExecOptions {
    //                                     detach: true,
    //                                     ..Default::default()
    //                                 }),
    //                             )
    //                             .await?
    //                         {
    //                             while let Some(Ok(msg)) = output.next().await {
    //                                 print!("{}", msg);
    //                             }
    //                         } else {
    //                             unreachable!();
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //         common::websocket::OpCodes::Hello => websocket.send(WebsocketMessage {
    //             op: OpCodes::Identify,
    //             event: Some(Box::new(Identify {
    //                 name: "runner1".to_string(),
    //                 password: "runner1234".to_string(),
    //             })),
    //         })?,
    //         common::websocket::OpCodes::HeartBeat => todo!(),
    //         _ => {}
    //     }
    // }
    // let lxc = LXC::new("fox-is-cute")?;
    // lxc.create(Some("alpine"), Some("edge")).unwrap();

    // let docker = Docker::new_async("fox-is-cute").await?;
    // docker.create_async("alpine", "latest").await.unwrap();

    Ok(())
}
