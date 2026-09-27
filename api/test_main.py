from fastapi.testclient import TestClient
from main import app
import time
import uuid

client = TestClient(app)

def test_join_queue():
    player_id = str(uuid.uuid4())
    payload = {
        "id": player_id,
        "mmr": 1500,
        "region": "na-east",
        "latency": 50,
        "role": "Flex",
        "party_id": None,
        "behavior_score": 10000,
        "queued_at": int(time.time())
    }
    
    response = client.post("/queue/join", json=payload)
    assert response.status_code == 200
    assert response.json()["status"] == "success"

def test_leave_queue():
    player_id = str(uuid.uuid4())
    # Note: query parameters for leave_queue
    response = client.post(f"/queue/leave?player_id={player_id}&region=na-east")
    assert response.status_code == 200
    assert response.json()["status"] == "success"

def test_get_match_status():
    player_id = str(uuid.uuid4())
    response = client.get(f"/match/status/{player_id}")
    assert response.status_code == 200
    assert response.json()["status"] == "pending"
