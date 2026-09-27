"""
match_listener.py
-----------------
An async background task that subscribes to the Redis `matches` pub/sub channel.
The Rust engine publishes match JSON to this channel whenever a match is formed.
This relay then pushes the data into Firebase Realtime Database, which the
Next.js frontend listens to via the Firebase client SDK for real-time updates.
"""

import asyncio
import json
import os
import redis.asyncio as redis
from firebase_client import write_match_to_firebase, write_player_match_index


async def listen_for_matches() -> None:
    """Subscribe to Redis pub/sub and relay match events to Firebase."""
    redis_url = os.environ.get("REDIS_URL", "redis://localhost:6379")
    client = redis.from_url(redis_url, decode_responses=True)
    pubsub = client.pubsub()
    await pubsub.subscribe("matches")

    print("[MatchListener] Subscribed to Redis 'matches' channel — waiting for events...")

    try:
        async for message in pubsub.listen():
            if message["type"] != "message":
                continue

            try:
                match_data: dict = json.loads(message["data"])
                match_id: str = match_data.get("id", "unknown")

                print(f"[MatchListener] Match received: {match_id} | Region: {match_data.get('region')} | Quality: {match_data.get('quality_score', 0):.1f}%")

                # Run Firebase writes in a thread pool so we don't block the event loop
                loop = asyncio.get_running_loop()

                # Write full match data
                await loop.run_in_executor(
                    None,
                    write_match_to_firebase,
                    match_id,
                    match_data,
                )

                # Write per-player reverse-indexes so the frontend can listen per-player
                for team_key in ("team1", "team2"):
                    for player in match_data.get(team_key, {}).get("players", []):
                        player_id = player.get("id")
                        if player_id:
                            await loop.run_in_executor(
                                None,
                                write_player_match_index,
                                player_id,
                                match_id,
                            )

                print(f"[MatchListener] Match {match_id} written to Firebase ✅")

            except json.JSONDecodeError as e:
                print(f"[MatchListener] Failed to parse match JSON: {e}")
            except Exception as e:
                print(f"[MatchListener] Error processing match: {e}")

    except asyncio.CancelledError:
        print("[MatchListener] Shutting down...")
    finally:
        await pubsub.unsubscribe("matches")
        await client.aclose()
