use std::any::TypeId;

use anyhow::Result;
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use channel_runner::{
    backends::{Backend, Docker, LXC},
    config::CONFIG,
    Websocket,
};

use channel_common::{
    events::{CreateJobRun, Identify},
    websocket::{OpCodes, WebsocketMessage},
};
use futures_util::StreamExt;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Args::parse();
    let runner_token = cli.token;

    // sudo::escalate_if_needed().unwrap();

    // let docker = channel_runner::docker::prelude::DockerInner::new().await?;
    // let image = channel_runner::docker::images::Image::create_image(
    //     "https://github.com/pylon-sh/powerline.git",
    //     "test-imgae",
    //     "latset",
    //     &docker,
    // )
    // .await?;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();

    sudo::escalate_if_needed().unwrap();
    let websocket = Websocket::new(CONFIG.runner.name.to_string(), runner_token.clone()).await?;
    let (_, _, _) = websocket.start().await;

    let mut reader = websocket.reader.lock().await;
    while let Some(message) = reader.recv().await {
        match message.op {
            channel_common::websocket::OpCodes::EventCreate => {
                if let Some(event) = message.event {
                    let d_any = event.as_any();
                    if d_any.type_id() == TypeId::of::<CreateJobRun>() {
                        let Some(data) = d_any.downcast_ref::<CreateJobRun>() else {
                            continue;
                        };

                        let backend = match data.pipeline.backend.clone().unwrap() {
                            channel_common::models::PipelineBackend::Docker => "docker",
                            channel_common::models::PipelineBackend::LXC => "lxc",
                        };

                        let image = match data.pipeline.image.clone() {
                            Some(img) => img,
                            None => CONFIG
                                .runner
                                .defaults
                                .get(backend)
                                .unwrap()
                                .image
                                .clone()
                                .unwrap(),
                        };

                        let release = match data.pipeline.release.clone() {
                            Some(img) => img,
                            None => CONFIG
                                .runner
                                .defaults
                                .get(backend)
                                .unwrap()
                                .release
                                .clone()
                                .unwrap(),
                        };

                        /*****************************************************************************
                        THE FOLLOWING CODE WILL BE MOVED IN THE FUTURE. DO NOT DEPEND ON IT BEING HERE
                        *****************************************************************************/
                        // if &CONFIG.runner.backend.name.to_lowercase() == "docker" {
                        println!("4");
                        let container_name = format!("runner-{}-{}", "temp_value", data.job.id);
                        let docker = Docker::new_async(&container_name).await?;
                        println!("5");
                        let container = docker.create_async(&image, &release).await?;
                        let container_id = container.id;
                        docker
                            ._inner
                            .0
                            .start_container::<String>(&container_id, None)
                            .await?;
                        println!("6");
                        for step in &data.steps {
                            println!("** RUNNING STEP: {:?} (#{})", step.name, step.id);
                            let cmd = step.run.split(' ').collect::<Vec<&str>>();
                            let exec = docker
                                ._inner
                                .0
                                .create_exec(
                                    &container_id,
                                    CreateExecOptions::<&str> {
                                        cmd: Some(cmd),
                                        attach_stdout: Some(true),
                                        attach_stderr: Some(true),
                                        ..Default::default()
                                    },
                                )
                                .await?;
                            let output = docker
                                ._inner
                                .0
                                .start_exec(
                                    &exec.id,
                                    Some(StartExecOptions {
                                        detach: false,
                                        ..Default::default()
                                    }),
                                )
                                .await
                                .unwrap();

                            match output {
                                StartExecResults::Attached { mut output, .. } => {
                                    while let Some(Ok(msg)) = output.next().await {
                                        print!("{}", msg);
                                    }
                                }
                                _ => println!("WHAT: {output:?}"),
                            }
                        }
                        // }
                    }
                }
            }
            channel_common::websocket::OpCodes::Hello => websocket.send(WebsocketMessage {
                op: OpCodes::Identify,
                event: Some(Box::new(Identify {
                    name: "ibis1".to_string(),
                    token: runner_token.clone(),
                })),
            })?,
            _ => {}
        }
    }
    // let lxc = LXC::new("fox-is-cute")?;
    // lxc.create(Some("alpine"), Some("edge")).unwrap();

    // let docker = Docker::new_async("fox-is-cute").await?;
    // docker.create_async("alpine", "latest").await.unwrap();

    Ok(())
}
