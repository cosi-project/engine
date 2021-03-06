use clap::Clap;
use cosi::{
    spec::resource::{CreateOptions, CreateRequest, DestroyOptions, DestroyRequest},
    spec::resource::{Metadata, Resource, Spec},
    ResourceInstance,
};
use serde::Deserialize;
use std::fs;

#[derive(Clap)]
#[clap(version = "0.1.0")]
struct Opts {
    #[clap(short, long, default_value = "127.0.0.1:50000")]
    address: String,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = "0.1.0")]
    Create(Create),
    #[clap(version = "0.1.0")]
    Delete(Delete),
}

#[derive(Clap)]
struct Create {
    #[clap(short)]
    filename: String,
}

#[derive(Clap)]
struct Delete {
    #[clap(short)]
    filename: String,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Create(t) => {
            let filename = t.filename.as_str();

            let contents = fs::read_to_string(filename).expect("failed to read the resource");

            let mut client = cosi::machinery::runtime::client::connect_state_tcp(opts.address)
                .await
                .unwrap();

            println!("Applying {}", filename);

            for document in serde_yaml::Deserializer::from_str(&contents) {
                let value = serde_yaml::Value::deserialize(document).unwrap();

                let resource: ResourceInstance =
                    serde_yaml::from_value(value).expect("failed to deserialize YAML");

                println!("Creating {}: {}", resource.r#type, resource.id);

                let json = serde_json::to_string(&resource.spec).unwrap();
                let spec = json.as_str();

                let request = tonic::Request::new(CreateRequest {
                    resource: Some(Resource {
                        metadata: Some(Metadata {
                            version: resource.version,
                            r#type: resource.r#type,
                            namespace: resource.namespace,
                            id: resource.id,
                            phase: "running".to_owned(),
                            ..Default::default()
                        }),
                        spec: Some(Spec {
                            proto_spec: vec![],
                            yaml_spec: spec.to_owned(),
                        }),
                    }),
                    options: Some(CreateOptions {
                        owner: String::from("system"),
                    }),
                });

                let r = client.create(request).await;

                match r {
                    Ok(_) => println!("Created"),
                    Err(err) => match err.code() {
                        tonic::Code::AlreadyExists => println!("Already exists"),
                        _ => std::process::exit(1),
                    },
                }
            }
        }
        SubCommand::Delete(t) => {
            let filename = t.filename.as_str();

            let contents = fs::read_to_string(filename).expect("failed to read the resource");

            let mut client = cosi::machinery::runtime::client::connect_state_tcp(opts.address)
                .await
                .unwrap();

            println!("Deleting {}", filename);

            for document in serde_yaml::Deserializer::from_str(&contents) {
                let value = serde_yaml::Value::deserialize(document).unwrap();

                let resource: ResourceInstance =
                    serde_yaml::from_value(value).expect("failed to deserialize YAML");

                println!("Deleting {}: {}", resource.r#type, resource.id);

                let request = tonic::Request::new(DestroyRequest {
                    namespace: resource.namespace,
                    r#type: resource.r#type,
                    id: resource.id,
                    options: Some(DestroyOptions {
                        owner: String::from("system"),
                    }),
                });

                let r = client.destroy(request).await;

                match r {
                    Ok(_) => println!("Deleted"),
                    Err(err) => match err.code() {
                        tonic::Code::AlreadyExists => println!("Already exists"),
                        _ => std::process::exit(1),
                    },
                }
            }
        }
    };
}
