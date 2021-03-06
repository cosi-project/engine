use cosi::unix;
use nix::unistd::Pid;

#[tokio::main]
#[cfg(unix)]
async fn main() {
    tokio::spawn(async {
        println!("Listening on {:?}", cosi::consts::SOCKET_ENGINE.to_owned());

        let service = cosi::machinery::engine::v1alpha1::EngineService::default();

        service.serve(cosi::consts::SOCKET_ENGINE.to_owned()).await
    });

    let (tx, reaper) = unix::process::reaper::new();
    let r1 = reaper.clone();
    let r2 = reaper.clone();
    let r3 = reaper.clone();

    tokio::spawn(async move {
        match cosi::machinery::runtime::load(cosi::consts::SOCKET_ENGINE.to_owned(), r1) {
            Ok(_) => println!("Loaded runtime"),
            Err(err) => println!("Failed to load runtime: {:?}", err),
        };

        match cosi::machinery::generator::load(cosi::consts::SOCKET_ENGINE.to_owned(), r2) {
            Ok(generators) => {
                for generator in generators.iter() {
                    println!("Loaded {:?}", generator);
                }
            }
            Err(err) => println!("Failed to load generators: {:?}", err),
        };

        match cosi::machinery::plugin::load(cosi::consts::SOCKET_ENGINE.to_owned(), r3) {
            Ok(plugins) => {
                for plugin in plugins.iter() {
                    println!("Loaded {:?}", plugin);
                }
            }
            Err(err) => println!("Failed to load plugins: {:?}", err),
        };
    });

    cosi::unix::handle_signals(
        || {
            println!("Ignoring SIGHUP");

            false
        },
        || {
            println!("Ignoring SIGINT");

            false
        },
        || {
            println!("Ignoring SIGTERM");

            false
        },
        || {
            if let Err(err) = reaper.reap(Pid::from_raw(-1), &tx) {
                println!("Failed to reap: {:?}", err)
            };

            false
        },
    )
    .await;
}
