use tracing::Level;
use unleash_client::unleash::UnleashBuilder;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::WARN)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let unleash = UnleashBuilder::default().build(
        std::env::var("UNLEASH_URL").unwrap_or("http://localhost:4242".to_string()),
        "unleash-client-rust".to_string(),
        std::env::var("UNLEASH_TOKEN").expect("env variable `UNLEASH_TOKEN` should be set"),
    );

    tokio::select!(
        _ = unleash.start() => {},
        _ = async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            println!("{:#?}", unleash.resolve_all(&mut Default::default()).unwrap());

            unleash.stop();
        } => {}
    );
}
