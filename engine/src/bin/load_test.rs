use engine::models::{Player, Role};
use redis::AsyncCommands;
use std::env;
use std::time::Instant;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(redis_url).unwrap();
    let mut con = client.get_connection_manager().await.unwrap();

    let region = "na-east";
    let queue_key = format!("queue:{}", region);
    
    // Clear queue first
    let _: () = con.del(&queue_key).await.unwrap();

    println!("Generating 10,000 mock players for load testing...");
    let start = Instant::now();
    
    let mut pipe = redis::pipe();
    let mut p = pipe.atomic();
    
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for i in 0..10_000 {
        let player = Player {
            id: Uuid::new_v4().to_string(),
            mmr: 1500 + (i % 1000) as i32,
            region: region.to_string(),
            latency: (i % 100) as u32,
            role: Some(Role::Flex),
            party_id: None,
            behavior_score: 10000,
            queued_at: current_time - (i % 300) as u64, // Simulated queue times
        };

        let json = serde_json::to_string(&player).unwrap();
        p = p.hset(&queue_key, &player.id, json);

        // Execute in batches to not blow up memory
        if i > 0 && i % 1000 == 0 {
            let _: () = p.query_async(&mut con).await.unwrap();
            pipe = redis::pipe();
            p = pipe.atomic();
        }
    }
    
    // final batch
    let _: () = p.query_async(&mut con).await.unwrap();

    println!("Successfully pushed 10,000 players in {:?}", start.elapsed());
    println!("Start the engine via `cargo run` to see how fast they are processed!");
}
