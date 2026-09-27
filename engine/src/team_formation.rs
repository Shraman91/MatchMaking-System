use crate::models::{Player, Role, Team};

pub struct MatchConstraints {
    pub team_size: usize,
    pub max_mmr_diff: i32,
    // E.g., [Role::Tank, Role::Dps, Role::Dps, Role::Support, Role::Support]
    pub required_roles: Option<Vec<Role>>,
    pub allow_cross_region: bool,
}

impl Default for MatchConstraints {
    fn default() -> Self {
        Self {
            team_size: 5,
            max_mmr_diff: 100, // This is dynamic based on queue time
            required_roles: None,
            allow_cross_region: false,
        }
    }
}

pub fn validate_team_composition(team: &Team, constraints: &MatchConstraints) -> bool {
    if team.players.len() != constraints.team_size {
        return false;
    }
    
    // Check role constraints if they exist
    if let Some(req_roles) = &constraints.required_roles {
        let mut available_roles = req_roles.clone();
        for player in &team.players {
            if let Some(role) = &player.role {
                if let Some(pos) = available_roles.iter().position(|r| r == role) {
                    available_roles.remove(pos);
                } else if *role == Role::Flex && !available_roles.is_empty() {
                    // Flex can fill any remaining role
                    available_roles.pop();
                } else {
                    return false; // Player role doesn't fit required roles
                }
            } else {
                // If player has no role selected, we assume they are flex
                if !available_roles.is_empty() {
                    available_roles.pop();
                } else {
                    return false;
                }
            }
        }
    }

    true
}

// Simple greedy allocator for testing (this would be more advanced in reality)
pub fn form_teams(players: Vec<Player>, constraints: &MatchConstraints) -> Option<(Team, Team)> {
    if players.len() < constraints.team_size * 2 {
        return None;
    }

    // Sort players by MMR to easily split them into balanced teams
    let mut sorted_players = players;
    sorted_players.sort_by_key(|p| p.mmr);

    let mut t1 = Vec::new();
    let mut t2 = Vec::new();
    let mut sum_mmr_1 = 0;
    let mut sum_mmr_2 = 0;

    // Distribute players to balance MMR
    // We reverse so we place highest MMR players first
    for (i, p) in sorted_players.into_iter().rev().enumerate() {
        if i >= constraints.team_size * 2 {
            break; // We only need enough for 2 teams
        }

        if t1.len() < constraints.team_size && t2.len() < constraints.team_size {
            if sum_mmr_1 <= sum_mmr_2 {
                sum_mmr_1 += p.mmr;
                t1.push(p);
            } else {
                sum_mmr_2 += p.mmr;
                t2.push(p);
            }
        } else if t1.len() < constraints.team_size {
            sum_mmr_1 += p.mmr;
            t1.push(p);
        } else {
            sum_mmr_2 += p.mmr;
            t2.push(p);
        }
    }

    let team1 = Team {
        avg_mmr: if t1.is_empty() { 0 } else { sum_mmr_1 / t1.len() as i32 },
        players: t1,
    };
    
    let team2 = Team {
        avg_mmr: if t2.is_empty() { 0 } else { sum_mmr_2 / t2.len() as i32 },
        players: t2,
    };

    if validate_team_composition(&team1, constraints) && validate_team_composition(&team2, constraints) {
        Some((team1, team2))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_player(mmr: i32, role: Option<Role>) -> Player {
        Player {
            id: "1".to_string(),
            mmr,
            region: "NA".to_string(),
            latency: 20,
            role,
            party_id: None,
            behavior_score: 10000,
            queued_at: 0,
        }
    }

    #[test]
    fn test_form_teams_basic() {
        let mut players = Vec::new();
        for i in 0..10 {
            players.push(create_dummy_player(1000 + (i * 10), None));
        }

        let constraints = MatchConstraints::default();
        let result = form_teams(players, &constraints);
        assert!(result.is_some());
        
        let (t1, t2) = result.unwrap();
        assert_eq!(t1.players.len(), 5);
        assert_eq!(t2.players.len(), 5);
        assert!((t1.avg_mmr - t2.avg_mmr).abs() <= 20); // Teams should be balanced
    }
}
