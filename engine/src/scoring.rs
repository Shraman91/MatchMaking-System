use crate::models::Team;

pub struct ScoringWeights {
    pub mmr_weight: f64,
    pub latency_weight: f64,
    pub role_weight: f64,
    pub behavior_weight: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            mmr_weight: 1.0,
            latency_weight: 0.5,
            role_weight: 2.0,
            behavior_weight: 1.0,
        }
    }
}

pub fn calculate_match_score(team1: &Team, team2: &Team, weights: &ScoringWeights) -> f64 {
    // 1. MMR Difference
    let mmr_diff = (team1.avg_mmr - team2.avg_mmr).abs() as f64;
    
    // 2. Latency (Average across all players in the match)
    let total_latency: u32 = team1.players.iter().chain(team2.players.iter()).map(|p| p.latency).sum();
    let num_players = (team1.players.len() + team2.players.len()) as f64;
    let avg_latency = if num_players > 0.0 {
        total_latency as f64 / num_players
    } else {
        0.0
    };

    // 3. Role Balance Penalty (Optional, for now just 0 if all constraints met, else penalty)
    // Constraint validation handles strict role limits, but we could penalize non-ideal comps here.
    let role_penalty = 0.0;

    // 4. Behavior Score Penalty
    // Assuming behavior score is out of 10,000 (Dota style)
    let total_behavior: u32 = team1.players.iter().chain(team2.players.iter()).map(|p| p.behavior_score).sum();
    let avg_behavior = if num_players > 0.0 {
        total_behavior as f64 / num_players
    } else {
        10000.0
    };
    
    // Lower score is better (it's a cost function)
    let behavior_penalty = 10000.0 - avg_behavior;

    let score = (weights.mmr_weight * mmr_diff)
        + (weights.latency_weight * avg_latency)
        + (weights.role_weight * role_penalty)
        + (weights.behavior_weight * behavior_penalty);

    score
}

// Higher quality is closer to 100.0
pub fn calculate_match_quality(score: f64) -> f64 {
    // Basic conversion from a cost function to a percentage.
    // 0 cost = 100% quality.
    let max_acceptable_cost = 500.0; 
    let quality = 100.0 * (1.0 - (score / max_acceptable_cost));
    quality.clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Player;

    fn create_dummy_player(mmr: i32, latency: u32, behavior_score: u32) -> Player {
        Player {
            id: "1".to_string(),
            mmr,
            region: "NA".to_string(),
            latency,
            role: None,
            party_id: None,
            behavior_score,
            queued_at: 0,
        }
    }

    #[test]
    fn test_perfect_match() {
        let p1 = create_dummy_player(1000, 20, 10000);
        let p2 = create_dummy_player(1000, 20, 10000);
        
        let team1 = Team { players: vec![p1.clone()], avg_mmr: 1000 };
        let team2 = Team { players: vec![p2.clone()], avg_mmr: 1000 };

        let weights = ScoringWeights::default();
        let score = calculate_match_score(&team1, &team2, &weights);
        let quality = calculate_match_quality(score);

        // Latency penalty = 0.5 * 20 = 10
        // Score = 10
        assert_eq!(score, 10.0);
        assert!(quality > 90.0);
    }
}
