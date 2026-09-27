use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Role {
    Tank,
    Dps,
    Support,
    Flex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub mmr: i32,
    pub region: String,
    pub latency: u32, 
    pub role: Option<Role>,
    pub party_id: Option<String>,
    pub behavior_score: u32,
    pub queued_at: u64, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub players: Vec<Player>,
    pub avg_mmr: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub region: String,
    pub team1: Team,
    pub team2: Team,
    pub quality_score: f64,
    pub created_at: u64,
}
