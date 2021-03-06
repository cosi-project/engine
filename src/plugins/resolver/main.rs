pub static NAME: &str = "resolver";

// Resource.
pub static API: &str = "cosi.dev";
pub static VERSION: &str = "v1alpha1";
pub static KIND: &str = "Resolver";
pub static NAMESPACE: &str = "system";

#[tokio::main]
#[cfg(unix)]
async fn main() {
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
