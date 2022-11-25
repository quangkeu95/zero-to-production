use derive_builder::Builder;
use tracing::log::LevelFilter;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Builder)]
pub struct TracingOptions {
    #[builder(default = "LevelFilter::Info")]
    pub crate_level: LevelFilter,
    #[builder(default = "LevelFilter::Off")]
    pub tower_http_level: LevelFilter,
}

pub fn init_tracing(crate_name: String, options: TracingOptions) {
    let crate_level = options.crate_level.as_str().to_lowercase();
    let tower_http_level = options.tower_http_level.as_str().to_lowercase();

    let env_filter_level = format!(
        "{}={},tower_http={}",
        crate_name, crate_level, tower_http_level
    );

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| env_filter_level.into());
    let formatting_layer = BunyanFormattingLayer::new(
        "zero2prod".into(),
        // Output the formatted spans to stdout.
        std::io::stdout,
    );

    tracing_subscriber::registry()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .init();
}
