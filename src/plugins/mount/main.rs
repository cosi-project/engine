use cosi::{
    machinery::plugin,
    spec::engine::Mount,
    spec::runtime::{
        ControllerInput, ControllerInputKind, ControllerOutput, ControllerOutputKind,
        ReconcileEventsRequest, RegisterControllerRequest, RuntimeListRequest, StartRequest,
    },
};
use std::io::{stdin, Read};

pub static NAME: &str = "mount";

// Resource.
pub static API: &str = "cosi.dev";
pub static VERSION: &str = "v1alpha1";
pub static KIND: &str = "Mount";
pub static NAMESPACE: &str = "system";

#[tokio::main]
#[cfg(unix)]
async fn main() {
    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer).unwrap();

    let socket = buffer.as_str().to_owned();

    let r = plugin::register(socket.clone(), NAME.to_owned()).await;

    match r {
        Ok(_) => println!("Registered {}", NAME),
        Err(err) => match err.code() {
            tonic::Code::AlreadyExists => println!("Already registered"),
            _ => std::process::exit(1),
        },
    }

    let address = String::from("127.0.0.1:50000");

    let mut client = cosi::machinery::runtime::client::connect_runtime_tcp(address.clone())
        .await
        .unwrap();

    let inputs = vec![ControllerInput {
        kind: ControllerInputKind::Strong as i32,
        namespace: NAMESPACE.to_string(),
        r#type: KIND.to_string(),
        id: None,
    }];

    let outputs = vec![ControllerOutput {
        r#type: format!("{}Status", KIND.to_string()),
        kind: ControllerOutputKind::Shared as i32,
    }];

    let request = RegisterControllerRequest {
        controller_name: NAME.to_string(),
        outputs,
        inputs,
    };

    client.register_controller(request).await.unwrap();

    tokio::spawn(async {
        let mut client = cosi::machinery::runtime::client::connect_adapter_tcp(address)
            .await
            .unwrap();

        let request = ReconcileEventsRequest {
            controller_token: NAME.to_string(),
        };

        let mut stream = client.reconcile_events(request).await.unwrap().into_inner();

        while let Some(_event) = stream.message().await.unwrap() {
            let request = RuntimeListRequest {
                controller_token: NAME.to_string(),
                namespace: NAMESPACE.to_string(),
                r#type: KIND.to_string(),
            };

            let mut stream = client.list(request).await.unwrap().into_inner();
            while let Some(response) = stream.message().await.unwrap() {
                if let Some(resource) = response.resource {
                    // TODO: Read /proc/self/mountinfo.

                    if let Some(metadata) = resource.metadata {
                        println!("{:?}", metadata);
                        if metadata.r#type != "Mount" {
                            continue;
                        }
                    }

                    if let Some(spec) = resource.spec {
                        let mount: Mount = serde_yaml::from_str(&spec.yaml_spec).unwrap();
                        println!("{:?}", mount);
                    }
                }
            }
        }
    });

    client.start(StartRequest {}).await.unwrap();

    cosi::unix::handle_signals(
        || {
            std::process::exit(0);
        },
        || {
            std::process::exit(0);
        },
        || {
            std::process::exit(0);
        },
        || false,
    )
    .await;
}
