mod common;

use common::setup;
use cosi::machinery::plugin;
use cosi::spec::resource::{Metadata, Resource, Spec};
use cosi::spec::runtime::{
    ControllerOutput, ControllerOutputKind, RegisterControllerRequest, RuntimeCreateRequest,
};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::Runtime;

#[test]
fn engine() {
    let (tx, rx): (Sender<()>, Receiver<()>) = mpsc::channel();

    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let dir = setup().unwrap();

        let engine_path = dir.join("engine.sock");
        let engine_socket = engine_path.into_os_string().into_string().unwrap();

        let runtime_path = dir.join("runtime.sock");
        let runtime_socket = runtime_path.into_os_string().into_string().unwrap();

        let e1 = engine_socket.clone();

        // Start the engine.

        tokio::spawn(async {
            println!("Engine: {}", e1);

            let service = cosi::machinery::engine::v1alpha1::EngineService::default();
            service.serve(e1).await;

            panic!("expected engine server to not fail")
        });

        // Start the runtime.

        let r1 = runtime_socket.clone();

        tokio::spawn(async {
            println!("Runtime: {}", r1);

            let service = cosi::machinery::runtime::v1alpha1::RuntimeService::default();
            service.serve(r1).await;

            panic!("expected runtime server to not fail")
        });

        // Register the plugin with the engine.

        tokio::spawn(async {
            std::thread::sleep(std::time::Duration::from_millis(500));

            let result = plugin::register(engine_socket, String::from("test")).await;

            assert!(result.is_ok())
        });

        // Register the controller with the runtime.

        let r2 = runtime_socket.clone();

        tokio::spawn(async {
            let mut client = cosi::machinery::runtime::client::connect_runtime(r2)
                .await
                .unwrap();

            let request = RegisterControllerRequest {
                controller_name: String::from("test"),
                outputs: vec![ControllerOutput {
                    r#type: String::from("TestStatus"),
                    kind: ControllerOutputKind::Shared as i32,
                }],
                inputs: vec![],
            };

            let result = client.register_controller(request).await;

            assert!(result.is_ok());

            println!("{:?}", result.unwrap());
        });

        // Crate a resource.

        tokio::spawn(async move {
            std::thread::sleep(std::time::Duration::from_millis(2000));

            let mut client = cosi::machinery::runtime::client::connect_adapter(runtime_socket)
                .await
                .unwrap();

            let request = RuntimeCreateRequest {
                controller_token: String::from("test"),
                resource: Some(Resource {
                    metadata: Some(Metadata {
                        owner: String::from("system"),
                        namespace: String::from("system"),
                        id: String::from("test"),
                        version: String::from("v1alpha1"),
                        phase: String::from("running"),
                        finalizers: vec![],
                        r#type: String::from("Test"),
                    }),
                    spec: Some(Spec {
                        proto_spec: vec![],
                        yaml_spec: String::from(""),
                    }),
                }),
            };

            let result = client.create(request).await;
            //assert!(result.is_ok());

            println!("{:?}", result.unwrap());

            assert!(tx.send(()).is_ok());
        });
    });

    rx.recv().unwrap();
}
