use redis::{AsyncCommands, Client, RedisError};
use serde_json;

use crate::models::{Player, Match};

pub struct RedisClient {
    #[allow(dead_code)]
    client: Client,
    pub connection_manager: redis::aio::ConnectionManager,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        let connection_manager = client.get_connection_manager().await?;
        Ok(Self {
            client,
            connection_manager,
        })
    }

    /// Try to acquire a lock for a specific region. Returns true if acquired.
    pub async fn acquire_lock(&mut self, region: &str, ttl_ms: u64) -> Result<bool, RedisError> {
        let lock_key = format!("lock:region:{}", region);
        let result: Option<String> = redis::cmd("SET")
            .arg(&lock_key)
            .arg("1")
            .arg("NX")
            .arg("PX")
            .arg(ttl_ms)
            .query_async(&mut self.connection_manager)
            .await?;
        
        Ok(result.is_some())
    }

    /// Release the lock for a specific region
    pub async fn release_lock(&mut self, region: &str) -> Result<(), RedisError> {
        let lock_key = format!("lock:region:{}", region);
        let _: () = self.connection_manager.del(&lock_key).await?;
        Ok(())
    }

    /// Pull all candidates from a region's queue
    /// In a real system, we might pull a subset if the queue is huge, but for now we pull all.
    pub async fn pull_candidates(&mut self, region: &str) -> Result<Vec<Player>, RedisError> {
        let queue_key = format!("queue:{}", region);
        // Assuming queue is a HASH map of player_id -> Player JSON
        let raw_players: std::collections::HashMap<String, String> = self.connection_manager.hgetall(&queue_key).await?;
        
        let mut players = Vec::new();
        for (_, player_json) in raw_players {
            if let Ok(player) = serde_json::from_str::<Player>(&player_json) {
                players.push(player);
            }
        }
        
        // Sort by queued_at to process oldest first
        players.sort_by_key(|p| p.queued_at);
        Ok(players)
    }

    pub async fn remove_players_from_queue(&mut self, region: &str, player_ids: &[String]) -> Result<(), RedisError> {
        if player_ids.is_empty() {
            return Ok(());
        }
        let queue_key = format!("queue:{}", region);
        let mut pipe = redis::pipe();
        let mut p = pipe.atomic();
        for pid in player_ids {
            p = p.hdel(&queue_key, pid);
        }
        let _: () = p.query_async(&mut self.connection_manager).await?;
        Ok(())
    }

    /// Publish a match event to the Redis pub/sub channel (consumed by Firebase relay)
    pub async fn publish_match(&mut self, match_data: &Match) -> Result<(), RedisError> {
        if let Ok(json) = serde_json::to_string(match_data) {
            let _: () = self.connection_manager.publish("matches", json).await?;
        }
        Ok(())
    }

    /// Persist the full match data at key `match:{id}` with a 1-hour TTL.
    /// This allows the FastAPI backend to serve match details directly from Redis.
    pub async fn save_match(&mut self, match_data: &Match) -> Result<(), RedisError> {
        if let Ok(json) = serde_json::to_string(match_data) {
            let key = format!("match:{}", match_data.id);
            let _: () = redis::cmd("SET")
                .arg(&key)
                .arg(&json)
                .arg("EX")
                .arg(3600u32)
                .query_async(&mut self.connection_manager)
                .await?;
        }
        Ok(())
    }

    /// Write a reverse-index `player_match:{player_id}` → `match_id` for O(1) status lookups
    pub async fn set_player_match_index(
        &mut self,
        player_id: &str,
        match_id: &str,
    ) -> Result<(), RedisError> {
        let key = format!("player_match:{}", player_id);
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(match_id)
            .arg("EX")
            .arg(3600u32)
            .query_async(&mut self.connection_manager)
            .await?;
        Ok(())
    }
}
