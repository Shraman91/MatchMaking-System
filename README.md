# Skill-Based Matchmaking System

A high-performance matchmaking system with a Rust engine and Python FastAPI backend.

## Setup

1. Copy `serviceAccountsKey.example.json` to `serviceAccountsKey.json` and fill in your Firebase credentials.
2. Run the stack:
   ```bash
   docker-compose up -d
   ```
3. Run the API:
   ```bash
   cd api
   pip install -r requirements.txt
   uvicorn main:app --reload
   ```
4. Run the Engine:
   ```bash
   cd engine
   cargo run
   ```
