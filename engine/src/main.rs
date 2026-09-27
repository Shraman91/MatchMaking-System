use engine::engine_loop::Engine;
use std::env;

#[tokio::main]
async fn main() {
    // Initialize logger — set RUST_LOG=info (or debug) to control verbosity
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    log::info!("Starting Matchmaking Engine...");

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let regions: Vec<String> = env::var("REGIONS")
        .unwrap_or_else(|_| "na-east,eu-west".to_string())
        .split(',')
        .map(|s| s.to_string())
        .collect();

    let engine = Engine {
        redis_url,
        regions,
    };

    engine.run().await;
}
