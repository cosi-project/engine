pub mod engine {
    pub mod client {
        use crate::spec::engine::engine_client::EngineClient;
        use std::convert::TryFrom;
        use tokio::net::UnixStream;
        use tonic::transport::{Endpoint, Uri};
        use tower::service_fn;

        pub async fn connect(
            socket: String,
        ) -> Result<EngineClient<tonic::transport::Channel>, Box<dyn std::error::Error>> {
            let channel = Endpoint::try_from("http://[::]")
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    UnixStream::connect(socket.clone())
                }))
                .await
                .unwrap();

            Ok(EngineClient::new(channel))
        }
    }

    pub mod v1alpha1 {
        use crate::{
            spec::engine::{
                engine_server::{Engine, EngineServer},
                Plugin, RegisterResponse,
            },
            unix,
        };
        use std::{
            fs,
            path::Path,
            sync::{Arc, Mutex},
        };
        use tonic::transport::Server;
        use tonic::{Code, Request, Response, Status};

        #[derive(Default, Clone)]
        pub struct EngineService {
            plugins: Arc<Mutex<Vec<Plugin>>>,
        }

        impl EngineService {
            pub async fn serve(self, socket: String) {
                if Path::new(&socket).exists() {
                    fs::remove_file(&socket).expect("failed to remove socket");
                }

                let s = socket.clone();

                let handle = tokio::spawn(async move {
                    match unix::UnixIncoming::bind(s) {
                        Ok(socket) => {
                            match Server::builder()
                                .add_service(EngineServer::new(self))
                                .serve_with_incoming(socket)
                                .await
                            {
                                Ok(_) => (),
                                Err(err) => println!("{:?}", err),
                            }
                        }
                        Err(err) => println!("{:?}", err),
                    }
                });

                handle.await.unwrap()
            }
        }

        #[tonic::async_trait]
        impl Engine for EngineService {
            async fn register(
                &self,
                request: Request<Plugin>,
            ) -> Result<Response<RegisterResponse>, Status> {
                let request_plugin = request.into_inner();

                let mutex = self.plugins.clone();
                let mut registered_plugins = mutex.lock().unwrap();

                for registered_plugin in registered_plugins.clone().into_iter() {
                    if registered_plugin.name == request_plugin.name {
                        return Err(Status::new(
                            Code::AlreadyExists,
                            format!("plugin is already registered: {:?}", request_plugin.name),
                        ));
                    }
                }

                registered_plugins.push(request_plugin);

                Ok(Response::new(RegisterResponse {}))
            }
        }
    }
}

pub mod runtime {
    use crate::consts;
    use crate::unix::process::reaper::Reaper;

    pub fn load(socket: String, reaper: Reaper) -> Result<(), Box<dyn std::error::Error>> {
        tokio::spawn(async {
            crate::unix::process::monitor(consts::RUNTIME.to_owned(), socket, reaper)
                .await
                .unwrap();
        });

        Ok(())
    }

    pub mod client {
        use crate::spec::resource::state_client::StateClient;
        use crate::spec::runtime::controller_adapter_client::ControllerAdapterClient;
        use crate::spec::runtime::controller_runtime_client::ControllerRuntimeClient;
        use std::convert::TryFrom;
        use tokio::net::UnixStream;
        use tonic::transport::{Endpoint, Uri};
        use tower::service_fn;

        pub async fn connect_state(
            socket: String,
        ) -> Result<StateClient<tonic::transport::Channel>, Box<dyn std::error::Error>> {
            let channel = Endpoint::try_from("http://[::]")
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    UnixStream::connect(socket.clone())
                }))
                .await
                .unwrap();

            Ok(StateClient::new(channel))
        }

        pub async fn connect_state_tcp(
            address: String,
        ) -> Result<StateClient<tonic::transport::Channel>, Box<dyn std::error::Error>> {
            let address = format!("http://{}", address);

            let client = StateClient::connect(address).await.unwrap();

            Ok(client)
        }

        pub async fn connect_runtime(
            socket: String,
        ) -> Result<ControllerRuntimeClient<tonic::transport::Channel>, Box<dyn std::error::Error>>
        {
            let channel = Endpoint::try_from("http://[::]")
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    UnixStream::connect(socket.clone())
                }))
                .await
                .unwrap();

            Ok(ControllerRuntimeClient::new(channel))
        }

        pub async fn connect_runtime_tcp(
            address: String,
        ) -> Result<ControllerRuntimeClient<tonic::transport::Channel>, Box<dyn std::error::Error>>
        {
            let address = format!("http://{}", address);

            let client = ControllerRuntimeClient::connect(address).await.unwrap();

            Ok(client)
        }

        pub async fn connect_adapter(
            socket: String,
        ) -> Result<ControllerAdapterClient<tonic::transport::Channel>, Box<dyn std::error::Error>>
        {
            let channel = Endpoint::try_from("http://[::]")
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    UnixStream::connect(socket.clone())
                }))
                .await
                .unwrap();

            Ok(ControllerAdapterClient::new(channel))
        }

        pub async fn connect_adapter_tcp(
            address: String,
        ) -> Result<ControllerAdapterClient<tonic::transport::Channel>, Box<dyn std::error::Error>>
        {
            let address = format!("http://{}", address);

            let client = ControllerAdapterClient::connect(address).await.unwrap();

            Ok(client)
        }
    }

    pub mod v1alpha1 {
        use crate::{
            spec::runtime::controller_adapter_server::{
                ControllerAdapter, ControllerAdapterServer,
            },
            spec::runtime::controller_runtime_server::{
                ControllerRuntime, ControllerRuntimeServer,
            },
            spec::runtime::{ReconcileEventsResponse, RuntimeListResponse},
            unix,
        };
        use futures::Stream;
        use std::pin::Pin;
        use std::{fs, path::Path};
        use tonic::{transport::Server, Request, Response, Status};

        #[derive(Default, Clone, Copy)]
        pub struct RuntimeService {}

        impl RuntimeService {}

        impl RuntimeService {
            pub async fn serve(self, socket: String) {
                if Path::new(&socket).exists() {
                    fs::remove_file(&socket).expect("failed to remove socket");
                }

                let s = socket.clone();

                let handle = tokio::spawn(async move {
                    match unix::UnixIncoming::bind(s) {
                        Ok(socket) => {
                            match Server::builder()
                                .add_service(ControllerRuntimeServer::new(self))
                                .add_service(ControllerAdapterServer::new(self))
                                .serve_with_incoming(socket)
                                .await
                            {
                                Ok(_) => (),
                                Err(err) => println!("{:?}", err),
                            }
                        }
                        Err(err) => println!("{:?}", err),
                    }
                });

                handle.await.unwrap()
            }
        }

        #[tonic::async_trait]
        impl ControllerRuntime for RuntimeService {
            async fn register_controller(
                &self,
                request: Request<crate::spec::runtime::RegisterControllerRequest>,
            ) -> Result<Response<crate::spec::runtime::RegisterControllerResponse>, Status>
            {
                let token = request.into_inner().controller_name;

                Ok(Response::new(
                    crate::spec::runtime::RegisterControllerResponse {
                        controller_token: token,
                    },
                ))
            }

            async fn start(
                &self,
                _request: Request<crate::spec::runtime::StartRequest>,
            ) -> Result<Response<crate::spec::runtime::StartResponse>, Status> {
                todo!()
            }

            async fn stop(
                &self,
                _request: Request<crate::spec::runtime::StopRequest>,
            ) -> Result<Response<crate::spec::runtime::StopResponse>, Status> {
                todo!()
            }
        }

        #[tonic::async_trait]
        impl ControllerAdapter for RuntimeService {
            type ReconcileEventsStream = Pin<
                Box<
                    dyn Stream<Item = Result<ReconcileEventsResponse, Status>>
                        + Send
                        + Sync
                        + 'static,
                >,
            >;

            async fn reconcile_events(
                &self,
                _request: Request<crate::spec::runtime::ReconcileEventsRequest>,
            ) -> Result<Response<Self::ReconcileEventsStream>, Status> {
                todo!()
            }

            async fn queue_reconcile(
                &self,
                _request: Request<crate::spec::runtime::QueueReconcileRequest>,
            ) -> Result<Response<crate::spec::runtime::QueueReconcileResponse>, Status>
            {
                todo!()
            }

            async fn update_inputs(
                &self,
                _request: Request<crate::spec::runtime::UpdateInputsRequest>,
            ) -> Result<Response<crate::spec::runtime::UpdateInputsResponse>, Status> {
                todo!()
            }

            async fn get(
                &self,
                _request: Request<crate::spec::runtime::RuntimeGetRequest>,
            ) -> Result<Response<crate::spec::runtime::RuntimeGetResponse>, Status> {
                todo!()
            }

            type ListStream = Pin<
                Box<dyn Stream<Item = Result<RuntimeListResponse, Status>> + Send + Sync + 'static>,
            >;

            async fn list(
                &self,
                _request: Request<crate::spec::runtime::RuntimeListRequest>,
            ) -> Result<Response<Self::ListStream>, Status> {
                todo!()
            }

            async fn watch_for(
                &self,
                _request: Request<crate::spec::runtime::RuntimeWatchForRequest>,
            ) -> Result<Response<crate::spec::runtime::RuntimeWatchForResponse>, Status>
            {
                todo!()
            }

            async fn create(
                &self,
                _request: Request<crate::spec::runtime::RuntimeCreateRequest>,
            ) -> Result<Response<crate::spec::runtime::RuntimeCreateResponse>, Status> {
                Ok(Response::new(
                    crate::spec::runtime::RuntimeCreateResponse {},
                ))
            }

            async fn update(
                &self,
                _request: Request<crate::spec::runtime::RuntimeUpdateRequest>,
            ) -> Result<Response<crate::spec::runtime::RuntimeUpdateResponse>, Status> {
                todo!()
            }

            async fn teardown(
                &self,
                _request: Request<crate::spec::runtime::RuntimeTeardownRequest>,
            ) -> Result<Response<crate::spec::runtime::RuntimeTeardownResponse>, Status>
            {
                todo!()
            }

            async fn destroy(
                &self,
                _request: Request<crate::spec::runtime::RuntimeDestroyRequest>,
            ) -> Result<Response<crate::spec::runtime::RuntimeDestroyResponse>, Status>
            {
                todo!()
            }

            async fn add_finalizer(
                &self,
                _request: Request<crate::spec::runtime::RuntimeAddFinalizerRequest>,
            ) -> Result<Response<crate::spec::runtime::RuntimeAddFinalizerResponse>, Status>
            {
                todo!()
            }

            async fn remove_finalizer(
                &self,
                _request: Request<crate::spec::runtime::RuntimeRemoveFinalizerRequest>,
            ) -> Result<Response<crate::spec::runtime::RuntimeRemoveFinalizerResponse>, Status>
            {
                todo!()
            }
        }
    }
}

pub mod generator {
    use crate::consts;
    use crate::unix::process::reaper::Reaper;
    use std::env;

    pub fn load(socket: String, reaper: Reaper) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let pattern = format!(
            "{}/*-{}-{}",
            consts::GENERATORS,
            env::consts::OS,
            env::consts::ARCH
        );

        super::load(socket, reaper, pattern)
    }
}

pub mod plugin {
    use crate::unix::process::reaper::Reaper;
    use crate::{
        consts,
        spec::engine::{Plugin, RegisterResponse},
    };
    use std::env;

    pub fn load(socket: String, reaper: Reaper) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let pattern = format!(
            "{}/*-{}-{}",
            consts::PLUGINS,
            env::consts::OS,
            env::consts::ARCH
        );

        super::load(socket, reaper, pattern)
    }

    pub async fn register(
        socket: String,
        name: String,
    ) -> Result<tonic::Response<RegisterResponse>, tonic::Status> {
        let mut client = super::engine::client::connect(socket).await.unwrap();

        let request = tonic::Request::new(Plugin { name });

        client.register(request).await
    }
}

use crate::unix::process::reaper::Reaper;
use glob::glob_with;
use glob::MatchOptions;

pub fn load(
    socket: String,
    reaper: Reaper,
    pattern: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let options = MatchOptions {
        case_sensitive: true,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let mut loaded: Vec<String> = vec![];

    for entry in glob_with(&pattern, options).unwrap() {
        if let Ok(path) = entry {
            loaded.push(path.display().to_string());

            let s = socket.clone();

            if let Ok(executable) = path.into_os_string().into_string() {
                let r = reaper.clone();

                tokio::spawn(async {
                    crate::unix::process::monitor(executable, s, r)
                        .await
                        .unwrap();
                });
            };
        }
    }

    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use crate::*;
}
