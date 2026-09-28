# Skill-Based Matchmaking System

A high-performance, skill-based matchmaking system for competitive games, in the spirit of Valorant, CS and Dota. A **Rust engine** does the matching, a **Python FastAPI** service exposes the player-facing API, **Redis** holds queue state, and **Firebase** pushes real-time match notifications to clients.

The goal is fair, low-latency, role-balanced matches: players of similar skill, on good connections, with sensible team compositions, found quickly.

## Table of Contents

- [Features](#features)
- [Architecture](#architecture)
- [How matching works](#how-matching-works)
- [Project structure](#project-structure)
- [Getting started](#getting-started)
- [API reference](#api-reference)
- [Testing](#testing)
- [Roadmap](#roadmap)

## Features

- **Rust matching engine** — the hot path (bucketing, scoring, team formation) is written in Rust for speed and predictable latency.
- **MMR bucket queues** — players are grouped by skill so candidate lookups stay cheap.
- **Dynamic range expansion** — the acceptable MMR gap widens the longer a player waits, trading a little match quality for shorter queue times.
- **Multi-factor scoring** — candidate matches are scored on MMR difference, latency, role balance and behavior score.
- **Role-based team composition** — teams are built to satisfy team-size, role and region constraints.
- **Region-aware queues** — players are matched within a region, with region locking so workers don't collide.
- **Horizontally scalable** — Redis locks and atomic operations let multiple engine workers run side by side.
- **FastAPI backend** — a small REST API for joining and leaving the queue and checking match status.
- **Real-time notifications** — when a match forms, it is published through Redis pub/sub and relayed to Firebase so clients update instantly, without polling.
- **Load-test harness** — pushes 10,000 mock players through the engine to measure throughput.
- **Tested** — unit tests for the pure matching logic (Rust) and integration tests for the API (pytest).

## Architecture

```
┌──────────┐  POST /queue/join   ┌──────────────┐  push player   ┌─────────┐
│  Client  │ ──────────────────► │   FastAPI    │ ─────────────► │  Redis  │
│          │                     │   backend    │                │ (queues)│
└────▲─────┘                     └──────────────┘                └────┬────┘
     │                                                                │ poll + lock
     │ real-time match event                                          ▼
┌────┴─────┐   Firebase Admin    ┌──────────────┐  pub/sub match  ┌─────────┐
│ Firebase │ ◄────────────────── │ match_listener│ ◄───────────── │  Rust   │
│          │                     │   (relay)    │                 │ engine  │
└──────────┘                     └──────────────┘                 └─────────┘
```

1. A client calls the API to **join the queue**; the backend writes the player into Redis.
2. The **Rust engine** continuously pulls candidates per region, locks them, and forms matches.
3. Formed matches are **published to Redis**.
4. A **relay process** (`match_listener.py`) subscribes to those events and writes them to **Firebase**, which pushes the update to the client.

## How matching works

**1. Bucketing.** Waiting players are kept in MMR buckets so the engine only compares players who are plausibly compatible.

**2. Range expansion.** Each player's acceptable MMR window grows with time in queue:

| Time in queue | Search window |
| --- | --- |
| 0–5 s | Tight — highest-quality matches only |
| 5–10 s | Widened |
| 10 s+ | Widest — prioritizes finding a game |

**3. Scoring.** Each candidate group is scored across several factors:

- **MMR difference** — how evenly matched the players are
- **Latency** — ping to the region/server
- **Role balance** — whether each team can field a valid composition
- **Behavior score** — keeps very poor-behavior players from being paired with everyone else

**4. Constraint validation and team formation.** Groups must satisfy team size, role composition and region constraints. Valid groups are split into balanced teams.

**5. Publish.** The final match is written to Redis and announced so the player and backend can react.

To run several workers safely, the engine uses Redis locks and atomic operations so that two workers never claim the same player.

## Project structure

```
MatchMaking-System/
├── api/                   # FastAPI backend (queue + match endpoints, Redis client, Firebase relay)
├── engine/                # Rust matching engine (scoring, buckets, team formation, Redis I/O)
├── docker-compose.yml     # Redis (with persistence and a healthcheck)
├── pytest.ini             # pytest configuration
└── implementation_plan.md # original design and phased build plan
```

## Getting started

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Python 3.10+
- A [Firebase](https://console.firebase.google.com/) project with a service account (for real-time notifications)

### 1. Clone the repository

```bash
git clone https://github.com/Shraman91/MatchMaking-System.git
cd MatchMaking-System
```

### 2. Configure Firebase

Copy `serviceAccountsKey.example.json` to `serviceAccountsKey.json` and fill in your Firebase service-account credentials.

> ⚠️ Never commit `serviceAccountsKey.json`. Make sure it is listed in `.gitignore`.

### 3. Start Redis

```bash
docker-compose up -d
```

This starts Redis on port `6379` with append-only persistence.

### 4. Run the API

```bash
cd api
pip install -r requirements.txt
uvicorn main:app --reload
```

The API is served at `http://127.0.0.1:8000`, with interactive docs at `http://127.0.0.1:8000/docs`.

### 5. Run the engine

In a new terminal:

```bash
cd engine
cargo run --release
```

### 6. Try it out

Join the queue with a few players and watch the engine logs form matches:

```bash
curl -X POST http://127.0.0.1:8000/queue/join \
  -H "Content-Type: application/json" \
  -d '{"player_id": "p1", "mmr": 1500, "region": "asia", "role": "duelist"}'
```

Then check status:

```bash
curl http://127.0.0.1:8000/match/status/p1
```

> The request body above is an example — use the fields defined by the `Player` model in `api/`.

## API reference

| Method | Endpoint | Description |
| --- | --- | --- |
| `POST` | `/queue/join` | Add a player to the matchmaking queue. |
| `POST` | `/queue/leave` | Remove a player from the queue. |
| `GET` | `/match/status/{player_id}` | Get a player's current queue/match status. |
| `GET` | `/match/{match_id}` | Get the details of a formed match. |

## Testing

```bash
# Rust engine — unit tests for scoring, bucket expansion and team formation
cd engine
cargo test

# API — integration tests
pytest
```

## Roadmap

- [ ] Party support (party integrity and party-vs-party matching)
- [ ] Smurf / skill-anomaly detection
- [ ] Match quality score surfaced to players
- [ ] Requeue logic for dodged or failed matches
- [ ] Sharded regional queues for larger scale
- [ ] Analytics dashboard
- [ ] Frontend "Finding Game" experience with live queue timer
