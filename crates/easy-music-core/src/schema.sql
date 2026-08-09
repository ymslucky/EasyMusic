-- EasyMusic SQLite schema
-- Executed inside a single transaction on first DB open.

-- Key-value store for metadata (last_scanned, schema version, …)
CREATE TABLE IF NOT EXISTS kv (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS artists (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS albums (
    id        TEXT PRIMARY KEY,
    title     TEXT NOT NULL,
    artist_id TEXT REFERENCES artists(id) ON DELETE SET NULL,
    year      INTEGER,
    genre     TEXT,
    UNIQUE (title, artist_id)
);

CREATE TABLE IF NOT EXISTS tracks (
    id            TEXT PRIMARY KEY,
    title         TEXT NOT NULL,
    artist_id     TEXT REFERENCES artists(id) ON DELETE SET NULL,
    album_id      TEXT REFERENCES albums(id)  ON DELETE SET NULL,
    genre         TEXT,
    path          TEXT NOT NULL UNIQUE,
    duration_secs INTEGER NOT NULL DEFAULT 0,
    track_number  INTEGER,
    year          INTEGER,
    file_format   TEXT
);

CREATE INDEX IF NOT EXISTS idx_tracks_artist  ON tracks (artist_id);
CREATE INDEX IF NOT EXISTS idx_tracks_album   ON tracks (album_id);
CREATE INDEX IF NOT EXISTS idx_tracks_title   ON tracks (title);
CREATE INDEX IF NOT EXISTS idx_tracks_genre   ON tracks (genre);

CREATE TABLE IF NOT EXISTS playlists (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist_tracks (
    playlist_id TEXT NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    TEXT NOT NULL REFERENCES tracks(id)     ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, track_id)
);

CREATE INDEX IF NOT EXISTS idx_playlist_tracks_order
    ON playlist_tracks (playlist_id, position);
