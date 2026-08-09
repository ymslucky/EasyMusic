//! SQLite persistence layer for the music library.
//!
//! Uses `rusqlite` (bundled SQLite) so there are no external system deps.
//! The schema is a classic normalized music-DB: tracks reference albums
//! and artists by id; playlists hold an ordered list of track ids via a
//! junction table.

use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{CoreError, CoreResult};
use crate::models::{
    Album, Artist, LibraryMetadata, Playlist, PlaylistWithTracks, ScanResult, Track,
};

/// Bundles the schema-creation statements, executed inside a single
/// transaction on first open.
const SCHEMA: &str = include_str!("schema.sql");

/// Thread-safe wrapper around a `rusqlite::Connection`.
///
/// The `Mutex` allows the whole library manager to live behind a single
/// `tauri::State` slot while still being safely shared across command threads.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open (or create) a database at `path`. If `path` is `:memory:` the
    /// database lives in RAM and vanishes when the handle is dropped.
    pub fn open(path: &str) -> CoreResult<Self> {
        let conn = Connection::open(path)?;
        // Good defaults for a local single-writer app.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(SCHEMA)?;
        tx.commit()?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Create an in-memory database — primarily for tests.
    pub fn open_memory() -> CoreResult<Self> {
        let db = Self::open(":memory:")?;
        // WAL not supported for :memory: — silently flip back.
        let _ = db
            .conn
            .lock()
            .unwrap()
            .pragma_update(None, "journal_mode", "MEMORY");
        Ok(db)
    }

    /// Acquire the connection lock and run `f` against the raw connection.
    ///
    /// This is the single choke-point through which all queries flow, so the
    /// `Mutex` is never accidentally left unlocked.
    pub fn with_conn<F, T>(&self, f: F) -> CoreResult<T>
    where
        F: FnOnce(&Connection) -> CoreResult<T>,
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }

    // -- track upsert ---------------------------------------------------

    /// Insert a new track, or update an existing one keyed on its file path
    /// (so rescans refresh metadata instead of duplicating rows).
    pub fn upsert_track(&self, track: &Track) -> CoreResult<()> {
        self.with_conn(|conn| {
            // Resolve or create the artist and album rows first.
            let artist_id = upsert_artist_row(conn, &track.artist)?;
            let album_id = track
                .album
                .as_deref()
                .map(|a| {
                    upsert_album_row(
                        conn,
                        a,
                        track.artist.as_str(),
                        track.year,
                        track.genre.as_deref(),
                    )
                })
                .transpose()?;

            conn.execute(
                "INSERT INTO tracks (id, title, artist_id, album_id, genre, path, duration_secs,
                                     track_number, year, file_format)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(path) DO UPDATE SET
                     title = excluded.title,
                     artist_id = excluded.artist_id,
                     album_id = excluded.album_id,
                     genre = excluded.genre,
                     duration_secs = excluded.duration_secs,
                     track_number = excluded.track_number,
                     year = excluded.year,
                     file_format = excluded.file_format",
                params![
                    track.id,
                    track.title,
                    artist_id,
                    album_id,
                    track.genre,
                    track.path,
                    track.duration_secs,
                    track.track_number,
                    track.year,
                    track.file_format,
                ],
            )?;
            Ok(())
        })
    }

    /// Bulk-upsert a batch of tracks inside a single transaction.
    /// Returns the scan stats (added vs updated vs skipped) by checking
    /// whether each path already existed before the upsert.
    pub fn upsert_tracks_batch(&self, tracks: &[Track]) -> CoreResult<ScanResult> {
        self.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            let mut added = 0u32;
            let mut updated = 0u32;
            let skipped = 0u32;
            for t in tracks {
                let existed: bool = tx
                    .query_row(
                        "SELECT 1 FROM tracks WHERE path = ?1",
                        params![t.path],
                        |_| Ok(()),
                    )
                    .optional()?
                    .is_some();
                let artist_id = upsert_artist_row(&tx, &t.artist)?;
                let album_id = t
                    .album
                    .as_deref()
                    .map(|a| {
                        upsert_album_row(&tx, a, t.artist.as_str(), t.year, t.genre.as_deref())
                    })
                    .transpose()?;
                tx.execute(
                    "INSERT INTO tracks (id, title, artist_id, album_id, genre, path, duration_secs,
                                         track_number, year, file_format)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                     ON CONFLICT(path) DO UPDATE SET
                         title = excluded.title,
                         artist_id = excluded.artist_id,
                         album_id = excluded.album_id,
                         genre = excluded.genre,
                         duration_secs = excluded.duration_secs,
                         track_number = excluded.track_number,
                         year = excluded.year,
                         file_format = excluded.file_format",
                    params![
                        t.id,
                        t.title,
                        artist_id,
                        album_id,
                        t.genre,
                        t.path,
                        t.duration_secs,
                        t.track_number,
                        t.year,
                        t.file_format,
                    ],
                )?;
                if existed {
                    updated += 1;
                } else {
                    added += 1;
                }
            }
            let _ = skipped; // reserved for future filter logic
            tx.commit()?;
            Ok(ScanResult {
                scanned_files: tracks.len() as u32,
                added,
                updated,
                skipped,
                errors: 0,
            })
        })
    }

    // -- track reads ----------------------------------------------------

    pub fn get_track(&self, id: &str) -> CoreResult<Option<Track>> {
        self.with_conn(|conn| {
            let result = conn.query_row(
                "SELECT t.id, t.title, ar.name, al.title, t.genre, t.path, t.duration_secs,
                        t.track_number, t.year, t.file_format
                 FROM tracks t
                 LEFT JOIN artists ar ON ar.id = t.artist_id
                 LEFT JOIN albums al ON al.id = t.album_id
                 WHERE t.id = ?1",
                params![id],
                row_to_track,
            );
            row_to_track_opt(result)
        })
    }

    pub fn all_tracks(&self) -> CoreResult<Vec<Track>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.id, t.title, ar.name, al.title, t.genre, t.path, t.duration_secs,
                        t.track_number, t.year, t.file_format
                 FROM tracks t
                 LEFT JOIN artists ar ON ar.id = t.artist_id
                 LEFT JOIN albums al ON al.id = t.album_id
                 ORDER BY t.title",
            )?;
            let rows = stmt.query_map([], row_to_track)?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    pub fn search_tracks(&self, query: &str) -> CoreResult<Vec<Track>> {
        let pattern = format!("%{query}%");
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.id, t.title, ar.name, al.title, t.genre, t.path, t.duration_secs,
                        t.track_number, t.year, t.file_format
                 FROM tracks t
                 LEFT JOIN artists ar ON ar.id = t.artist_id
                 LEFT JOIN albums al ON al.id = t.album_id
                 WHERE t.title LIKE ?1 OR ar.name LIKE ?1 OR al.title LIKE ?1 OR t.genre LIKE ?1
                 ORDER BY t.title",
            )?;
            let rows = stmt.query_map(params![pattern], row_to_track)?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    pub fn filter_tracks(&self, filter: &crate::models::TrackFilter) -> CoreResult<Vec<Track>> {
        self.with_conn(|conn| {
            let mut sql = String::from(
                "SELECT t.id, t.title, ar.name, al.title, t.genre, t.path, t.duration_secs,
                        t.track_number, t.year, t.file_format
                 FROM tracks t
                 LEFT JOIN artists ar ON ar.id = t.artist_id
                 LEFT JOIN albums al ON al.id = t.album_id WHERE 1=1",
            );
            let mut p: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            if let Some(a) = &filter.artist {
                sql.push_str(" AND ar.name = ?");
                p.push(Box::new(a.clone()));
            }
            if let Some(b) = &filter.album {
                sql.push_str(" AND al.title = ?");
                p.push(Box::new(b.clone()));
            }
            if let Some(g) = &filter.genre {
                sql.push_str(" AND t.genre = ?");
                p.push(Box::new(g.clone()));
            }
            if let Some(min) = filter.min_duration_secs {
                sql.push_str(" AND t.duration_secs >= ?");
                p.push(Box::new(min));
            }
            if let Some(max) = filter.max_duration_secs {
                sql.push_str(" AND t.duration_secs <= ?");
                p.push(Box::new(max));
            }
            sql.push_str(" ORDER BY t.title");
            let mut stmt = conn.prepare(&sql)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> = p.iter().map(|b| b.as_ref()).collect();
            let rows = stmt.query_map(params_refs.as_slice(), row_to_track)?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    // -- albums / artists ----------------------------------------------

    pub fn all_albums(&self) -> CoreResult<Vec<Album>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT al.id, al.title, ar.name, al.year, al.genre,
                        COUNT(t.id) AS track_count
                 FROM albums al
                 LEFT JOIN tracks t ON t.album_id = al.id
                 LEFT JOIN artists ar ON ar.id = al.artist_id
                 GROUP BY al.id
                 ORDER BY al.title",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(Album {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    year: row.get(3)?,
                    genre: row.get(4)?,
                    track_count: row.get::<_, i64>(5)? as u32,
                })
            })?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    pub fn all_artists(&self) -> CoreResult<Vec<Artist>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT ar.id, ar.name,
                        COUNT(DISTINCT al.id) AS album_count,
                        COUNT(t.id) AS track_count
                 FROM artists ar
                 LEFT JOIN albums al ON al.artist_id = ar.id
                 LEFT JOIN tracks t ON t.artist_id = ar.id
                 GROUP BY ar.id
                 ORDER BY ar.name",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(Artist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    album_count: row.get::<_, i64>(2)? as u32,
                    track_count: row.get::<_, i64>(3)? as u32,
                })
            })?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    // -- metadata -------------------------------------------------------

    pub fn library_metadata(&self) -> CoreResult<LibraryMetadata> {
        self.with_conn(|conn| {
            let total_tracks: i64 =
                conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))?;
            let total_albums: i64 =
                conn.query_row("SELECT COUNT(*) FROM albums", [], |row| row.get(0))?;
            let total_artists: i64 =
                conn.query_row("SELECT COUNT(*) FROM artists", [], |row| row.get(0))?;
            let total_playlists: i64 =
                conn.query_row("SELECT COUNT(*) FROM playlists", [], |row| row.get(0))?;
            let total_duration_secs: i64 = conn.query_row(
                "SELECT COALESCE(SUM(duration_secs), 0) FROM tracks",
                [],
                |row| row.get(0),
            )?;
            let last_scanned: Option<String> = conn
                .query_row(
                    "SELECT value FROM kv WHERE key = 'last_scanned'",
                    [],
                    |row| row.get(0),
                )
                .ok();
            Ok(LibraryMetadata {
                total_tracks: total_tracks as u32,
                total_albums: total_albums as u32,
                total_artists: total_artists as u32,
                total_playlists: total_playlists as u32,
                total_duration_secs: total_duration_secs as u64,
                last_scanned,
            })
        })
    }

    pub fn set_last_scanned(&self, iso_ts: &str) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO kv (key, value) VALUES ('last_scanned', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![iso_ts],
            )?;
            Ok(())
        })
    }

    // -- playlists ------------------------------------------------------

    pub fn create_playlist(&self, name: &str) -> CoreResult<Playlist> {
        if name.trim().is_empty() {
            return Err(CoreError::Invalid("playlist name cannot be empty".into()));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO playlists (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![id, name.trim(), now],
            )?;
            Ok(())
        })?;
        Ok(Playlist {
            id,
            name: name.trim().to_string(),
            track_count: 0,
            created_at: now,
        })
    }

    pub fn rename_playlist(&self, id: &str, new_name: &str) -> CoreResult<()> {
        if new_name.trim().is_empty() {
            return Err(CoreError::Invalid("playlist name cannot be empty".into()));
        }
        let affected = self.with_conn(|conn| {
            Ok(conn.execute(
                "UPDATE playlists SET name = ?1 WHERE id = ?2",
                params![new_name.trim(), id],
            )?)
        })?;
        if affected == 0 {
            return Err(CoreError::NotFound(format!("playlist '{id}'")));
        }
        Ok(())
    }

    pub fn delete_playlist(&self, id: &str) -> CoreResult<()> {
        let affected = self.with_conn(|conn| {
            Ok(conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?)
        })?;
        if affected == 0 {
            return Err(CoreError::NotFound(format!("playlist '{id}'")));
        }
        Ok(())
    }

    pub fn all_playlists(&self) -> CoreResult<Vec<Playlist>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT p.id, p.name, p.created_at,
                        COUNT(pt.track_id) AS track_count
                 FROM playlists p
                 LEFT JOIN playlist_tracks pt ON pt.playlist_id = p.id
                 GROUP BY p.id
                 ORDER BY p.name",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(Playlist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    track_count: row.get::<_, i64>(3)? as u32,
                })
            })?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    pub fn get_playlist(&self, id: &str) -> CoreResult<PlaylistWithTracks> {
        self.with_conn(|conn| {
            let playlist: Playlist = conn
                .query_row(
                    "SELECT p.id, p.name, p.created_at,
                            (SELECT COUNT(*) FROM playlist_tracks pt WHERE pt.playlist_id = p.id)
                     FROM playlists p WHERE p.id = ?1",
                    params![id],
                    |row| {
                        Ok(Playlist {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            created_at: row.get(2)?,
                            track_count: row.get::<_, i64>(3)? as u32,
                        })
                    },
                )
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => {
                        CoreError::NotFound(format!("playlist '{id}'"))
                    }
                    other => CoreError::Database(other.to_string()),
                })?;
            let mut stmt = conn.prepare(
                "SELECT t.id, t.title, ar.name, al.title, t.genre, t.path, t.duration_secs,
                        t.track_number, t.year, t.file_format
                 FROM playlist_tracks pt
                 JOIN tracks t ON t.id = pt.track_id
                 LEFT JOIN artists ar ON ar.id = t.artist_id
                 LEFT JOIN albums al ON al.id = t.album_id
                 WHERE pt.playlist_id = ?1
                 ORDER BY pt.position",
            )?;
            let rows = stmt.query_map(params![id], row_to_track)?;
            let tracks = rows.collect::<Result<Vec<_>, _>>()?;
            Ok(PlaylistWithTracks { playlist, tracks })
        })
    }

    pub fn add_track_to_playlist(&self, playlist_id: &str, track_id: &str) -> CoreResult<()> {
        self.with_conn(|conn| {
            // Validate both exist; FK columns have ON DELETE CASCADE but
            // we want a precise NotFound error.
            let playlist_exists: bool = conn
                .query_row(
                    "SELECT 1 FROM playlists WHERE id = ?1",
                    params![playlist_id],
                    |_| Ok(()),
                )
                .optional()?
                .is_some();
            if !playlist_exists {
                return Err(CoreError::NotFound(format!("playlist '{playlist_id}'")));
            }
            let track_exists: bool = conn
                .query_row("SELECT 1 FROM tracks WHERE id = ?1", params![track_id], |_| {
                    Ok(())
                })
                .optional()?
                .is_some();
            if !track_exists {
                return Err(CoreError::NotFound(format!("track '{track_id}'")));
            }
            let next_pos: i64 = conn.query_row(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_tracks WHERE playlist_id = ?1",
                params![playlist_id],
                |row| row.get(0),
            )?;
            conn.execute(
                "INSERT INTO playlist_tracks (playlist_id, track_id, position)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(playlist_id, track_id) DO NOTHING",
                params![playlist_id, track_id, next_pos],
            )?;
            Ok(())
        })
    }

    pub fn remove_track_from_playlist(&self, playlist_id: &str, track_id: &str) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
                params![playlist_id, track_id],
            )?;
            Ok(())
        })
    }
}

// ---- helpers ----------------------------------------------------------

/// Insert a new artist row or return the existing row's id.
fn upsert_artist_row(conn: &Connection, name: &str) -> CoreResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Invalid("artist name cannot be empty".into()));
    }
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM artists WHERE name = ?1",
            params![trimmed],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO artists (id, name) VALUES (?1, ?2)",
        params![id, trimmed],
    )?;
    Ok(id)
}

/// Insert a new album row or return the existing row's id.
#[allow(clippy::too_many_arguments)]
fn upsert_album_row(
    conn: &Connection,
    title: &str,
    artist: &str,
    year: Option<u32>,
    genre: Option<&str>,
) -> CoreResult<String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Invalid("album title cannot be empty".into()));
    }
    let artist_id = upsert_artist_row(conn, artist)?;
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM albums WHERE title = ?1 AND artist_id = ?2",
            params![trimmed, artist_id],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO albums (id, title, artist_id, year, genre)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, trimmed, artist_id, year, genre],
    )?;
    Ok(id)
}

/// Map one SELECT row (in the canonical 10-column track order) to a `Track`.
fn row_to_track(row: &rusqlite::Row<'_>) -> rusqlite::Result<Track> {
    Ok(Track {
        id: row.get(0)?,
        title: row.get(1)?,
        artist: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        album: row.get(3)?,
        genre: row.get(4)?,
        path: row.get(5)?,
        duration_secs: row.get::<_, i64>(6)? as u32,
        track_number: row.get::<_, Option<i64>>(7)?.map(|n| n as u32),
        year: row.get::<_, Option<i64>>(8)?.map(|n| n as u32),
        file_format: row.get(9)?,
    })
}

/// Convenience to turn a single-row query result into an `Option<Track>`.
fn row_to_track_opt(result: rusqlite::Result<Track>) -> CoreResult<Option<Track>> {
    match result {
        Ok(t) => Ok(Some(t)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(CoreError::Database(e.to_string())),
    }
}
