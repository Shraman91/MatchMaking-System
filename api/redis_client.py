import redis.asyncio as redis
from models import Player, Match
import json
from typing import Optional
import os

class RedisClient:
    def __init__(self):
        redis_url = os.environ.get("REDIS_URL", "redis://localhost:6379")
        self.redis = redis.from_url(redis_url, decode_responses=True)

    async def add_to_queue(self, player: Player) -> None:
        queue_key = f"queue:{player.region}"
        await self.redis.hset(queue_key, player.id, player.model_dump_json())

    async def remove_from_queue(self, player_id: str, region: str) -> None:
        queue_key = f"queue:{region}"
        await self.redis.hdel(queue_key, player_id)

    async def get_match(self, match_id: str) -> Optional[Match]:
        """Fetch the full match data persisted by the Rust engine at key `match:{match_id}`."""
        data = await self.redis.get(f"match:{match_id}")
        if not data:
            return None
        return Match.model_validate_json(data)

    async def get_player_match_id(self, player_id: str) -> Optional[str]:
        """Look up the match_id for a player via the reverse-index written by the Rust engine."""
        return await self.redis.get(f"player_match:{player_id}")


_redis_client = None

def get_redis_client() -> RedisClient:
    global _redis_client
    if _redis_client is None:
        _redis_client = RedisClient()
    return _redis_client
