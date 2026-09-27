import os
import firebase_admin
from firebase_admin import credentials, db

_app = None

def get_firebase_app() -> firebase_admin.App:
    """
    Returns the Firebase Admin app, initializing it on first call.
    Uses the service account key file at the project root.
    """
    global _app
    if _app is not None:
        return _app

    # Path to the service account key — place this relative to the api/ directory
    key_path = os.environ.get("FIREBASE_KEY_PATH")
    if not key_path:
        raise ValueError("FIREBASE_KEY_PATH environment variable is required")

    key_path = os.path.abspath(key_path)

    try:
        cred = credentials.Certificate(key_path)
        _app = firebase_admin.initialize_app(
            cred,
            {
                # Realtime Database URL
                "databaseURL": os.environ.get("FIREBASE_DB_URL")
            },
        )
    except Exception as e:
        print(f"Error initializing Firebase: {e}")
        raise
    return _app


def write_match_to_firebase(match_id: str, match_data: dict) -> None:
    """Write a full match payload to Firebase Realtime DB under `matches/{match_id}`."""
    get_firebase_app()
    db.reference(f"matches/{match_id}").set(match_data)


def write_player_match_index(player_id: str, match_id: str) -> None:
    """Write the reverse-index `player_matches/{player_id}` → match_id to Firebase."""
    get_firebase_app()
    db.reference(f"player_matches/{player_id}").set(match_id)
