from contextlib import asynccontextmanager
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from models import Player, Match
from redis_client import get_redis_client
from match_listener import listen_for_matches
from firebase_client import get_firebase_app
import time
import asyncio
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Initialize Firebase Admin SDK at startup
    get_firebase_app()
    # Start the Redis pub/sub → Firebase relay as a background task
    task = asyncio.create_task(listen_for_matches())
    logger.info("[API] Match listener started.")
    yield
    # Clean shutdown
    task.cancel()
    try:
        await task
    except asyncio.CancelledError:
        pass
    logger.info("[API] Match listener stopped.")


app = FastAPI(title="Matchmaking API", lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Adjust for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/health")
async def health_check():
    """Health check for API and Redis connectivity."""
    try:
        if await get_redis_client().redis.ping():
            return {"status": "ok", "redis": "connected"}
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Service unavailable: {e}")
    return {"status": "ok", "redis": "disconnected"}


@app.post("/queue/join")
async def join_queue(player: Player):
    """Add a player to the matchmaking queue for their region."""
    if player.queued_at is None:
        player.queued_at = int(time.time())
    await get_redis_client().add_to_queue(player)
    return {"status": "success", "message": f"Player {player.id} added to {player.region} queue"}


@app.post("/queue/leave")
async def leave_queue(player_id: str, region: str):
    """Remove a player from the matchmaking queue."""
    await get_redis_client().remove_from_queue(player_id, region)
    return {"status": "success", "message": f"Player {player.id} removed from {region} queue"}


@app.get("/match/status/{player_id}")
async def get_match_status(player_id: str):
    """
    Check if a player has been matched. The Rust engine writes a reverse-index
    `player_match:{player_id}` → match_id to Redis when a match is formed.
    """
    match_id = await get_redis_client().get_player_match_id(player_id)
    if match_id:
        return {"status": "matched", "match_id": match_id}
    return {"status": "pending"}


@app.get("/match/{match_id}", response_model=Match)
async def get_match(match_id: str):
    """Fetch full match data (teams, quality score, etc.) persisted by the Rust engine."""
    match = await get_redis_client().get_match(match_id)
    if not match:
        raise HTTPException(status_code=404, detail=f"Match {match_id} not found")
    return match
