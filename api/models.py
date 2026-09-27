from pydantic import BaseModel
from typing import Optional, List
from enum import Enum

class Role(str, Enum):
    Tank = "Tank"
    Dps = "Dps"
    Support = "Support"
    Flex = "Flex"

class Player(BaseModel):
    id: str
    mmr: int
    region: str
    latency: int
    role: Optional[Role] = None
    party_id: Optional[str] = None
    behavior_score: int
    queued_at: Optional[int] = None

class Team(BaseModel):
    players: List[Player]
    avg_mmr: int

class Match(BaseModel):
    id: str
    region: str
    team1: Team
    team2: Team
    quality_score: float
    created_at: int
