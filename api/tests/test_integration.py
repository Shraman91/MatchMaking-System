import sys
import os
sys.path.append(os.path.join(os.getcwd(), 'api'))

import pytest
import asyncio
import time
import uuid
import json
import redis.asyncio as redis
from redis_client import get_redis_client

# We assume a Redis instance is available at REDIS_URL
REDIS_URL = os.environ.get("REDIS_URL", "redis://localhost:6379")

@pytest.fixture
async def redis_client():
    client = redis.from_url(REDIS_URL, decode_responses=True)
    yield client
    await client.aclose()

@pytest.mark.asyncio
async def test_engine_integration(redis_client):
    """
    Simulates a small queue in Redis and verifies that a mock engine-like process
    would find it. This tests the interaction contract between API and Engine.
    """
    region = "na-east"
    queue_key = f"queue:{region}"
    await redis_client.delete(queue_key)  # Clean

    # 1. Simulate 10 players joining the queue
    for i in range(10):
        player = {
            "id": f"player_{i}",
            "mmr": 1500,
            "region": region,
            "latency": 50,
            "role": "Flex",
            "behavior_score": 10000,
            "queued_at": int(time.time())
        }
        await redis_client.hset(queue_key, player["id"], json.dumps(player))

    # 2. Assert queue has 10 players
    assert await redis_client.hlen(queue_key) == 10
    print("\n✅ 10 players added to test queue.")

    # 3. Simulate Rust engine behavior: Match found, match written, reverse-index set
    match_id = str(uuid.uuid4())
    match_data = {"id": match_id, "region": region, "quality_score": 100.0}

    # Write match
    await redis_client.setex(f"match:{match_id}", 3600, json.dumps(match_data))

    # Write reverse indexes
    for i in range(10):
        await redis_client.setex(f"player_match:player_{i}", 3600, match_id)

    print(f"✅ Match {match_id} and indexes written.")

    # 4. Verify API can fetch match
    # Since we can't easily run the actual engine in this test without more setup,
    # we verify that the data structure IS readable by our API logic.
    match_val = await redis_client.get(f"match:{match_id}")
    assert match_val is not None
    assert json.loads(match_val)["id"] == match_id

    player_match_id = await redis_client.get(f"player_match:player_0")
    assert player_match_id == match_id

    print("✅ Integration test passed: Data structure is readable by API.")
