use std::time::Duration;
use crate::redis_client::RedisClient;
use crate::team_formation::{self, MatchConstraints};
use crate::bucket_expansion;
use crate::scoring::{self, ScoringWeights};
use crate::models::Match;
use log::{info, warn, error};

pub struct Engine {
    pub redis_url: String,
    pub regions: Vec<String>,
}

impl Engine {
pub async fn run(&self) {
        let mut client = match RedisClient::new(&self.redis_url).await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to Redis: {}", e);
                return;
            }
        };

        info!("Matchmaking Engine started. Watching regions: {:?}", self.regions);

        let shutdown = tokio::signal::ctrl_c();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    info!("Shutdown signal received. Exiting...");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    for region in &self.regions {
                        match client.acquire_lock(region, 2000).await {
                            Ok(true) => {
                                self.process_region(&mut client, region).await;
                                let _ = client.release_lock(region).await;
                            }
                            Ok(false) => {}
                            Err(e) => {
                                error!("Redis lock error for region {}: {}", region, e);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn process_region(&self, client: &mut RedisClient, region: &str) {
        // 1. Pull all candidates for this region, sorted by wait time (oldest first)
        let candidates = match client.pull_candidates(region).await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to pull candidates for {}: {}", region, e);
                return;
            }
        };

        if candidates.len() < 10 {
            return;
        }

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let weights = ScoringWeights::default();

        // 2. Skill-Based Matching:
        //    We attempt to build as many valid 5v5 matches as possible from the candidate pool.
        //    For each "anchor" player (the one who has waited the longest), we collect up to 9
        //    other players whose MMR falls within the dynamic expansion window for that anchor.
        let mut matched_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut formed_matches: Vec<Match> = Vec::new();

        let constraints = MatchConstraints::default();

        for anchor in &candidates {
            if matched_ids.contains(&anchor.id) {
                continue;
            }

            let anchor_wait = current_time.saturating_sub(anchor.queued_at);

            // Collect up to 9 compatible players for the anchor
            let mut pool: Vec<_> = vec![anchor.clone()];
            for candidate in &candidates {
                if candidate.id == anchor.id || matched_ids.contains(&candidate.id) {
                    continue;
                }
                // Both directions of MMR compatibility must be satisfied (fairest match)
                let cand_wait = current_time.saturating_sub(candidate.queued_at);
                let anchor_ok = bucket_expansion::is_within_range(anchor.mmr, candidate.mmr, anchor_wait);
                let cand_ok = bucket_expansion::is_within_range(candidate.mmr, anchor.mmr, cand_wait);

                if anchor_ok && cand_ok {
                    pool.push(candidate.clone());
                }

                if pool.len() == constraints.team_size * 2 {
                    break;
                }
            }

            // Need exactly 10 players (2x team_size) for a valid match
            if pool.len() < constraints.team_size * 2 {
                continue;
            }

            // 3. Form balanced teams using the SBMM team formation algorithm
            let pool_for_match = pool[..constraints.team_size * 2].to_vec();
            let pool_ids: Vec<String> = pool_for_match.iter().map(|p| p.id.clone()).collect();

            if let Some((team1, team2)) = team_formation::form_teams(pool_for_match, &constraints) {
                // 4. Score the match quality
                let score = scoring::calculate_match_score(&team1, &team2, &weights);
                let quality = scoring::calculate_match_quality(score);

                let match_id = uuid::Uuid::new_v4().to_string();
                let new_match = Match {
                    id: match_id.clone(),
                    region: region.to_string(),
                    team1,
                    team2,
                    quality_score: quality,
                    created_at: current_time,
                };

                info!(
                    "✅ Match {} formed in {} | Quality: {:.1}% | Players: {:?}",
                    match_id, region, quality, pool_ids
                );

                // 5. Persist match + player reverse-indexes in Redis
                if let Err(e) = client.save_match(&new_match).await {
                    error!("Failed to save match {}: {}", match_id, e);
                }
                for pid in &pool_ids {
                    if let Err(e) = client.set_player_match_index(pid, &match_id).await {
                        error!("Failed to set player_match index for {}: {}", pid, e);
                    }
                }

                // 6. Publish to pub/sub for the Firebase relay listener
                if let Err(e) = client.publish_match(&new_match).await {
                    error!("Failed to publish match {}: {}", match_id, e);
                }

                // Mark all these players as matched
                for pid in pool_ids {
                    matched_ids.insert(pid);
                }

                formed_matches.push(new_match);
            } else {
                warn!("team_formation failed for pool anchored at {} in {}", anchor.id, region);
            }
        }

        // 7. Remove all matched players from the queue in one sweep
        if !matched_ids.is_empty() {
            let ids: Vec<String> = matched_ids.into_iter().collect();
            if let Err(e) = client.remove_players_from_queue(region, &ids).await {
                error!("Failed to remove matched players from queue: {}", e);
            }
            info!("Removed {} matched players from {} queue", ids.len(), region);
        }
    }
}
