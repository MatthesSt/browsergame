//! The leaderboard itself: one best entry per player, kept in memory, mirrored to a
//! JSON file so a restart does not wipe the board, and broadcast to every connected
//! websocket whenever it changes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

/// How many entries a snapshot carries by default.
pub const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 100;
/// Names longer than this are rejected rather than truncated, so nobody is surprised
/// by what ends up on the board.
const MAX_NAME_LEN: usize = 24;
const MAX_PLAYER_ID_LEN: usize = 64;
/// A score this large can only come from a tampered client.
const MAX_SCORE: u64 = 1_000_000_000_000;
/// Lost broadcasts are recovered by re-sending a snapshot, so this only needs to
/// absorb a burst.
const BROADCAST_CAPACITY: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub player_id: String,
    pub name: String,
    pub score: u64,
    #[serde(default)]
    pub wave: u32,
    /// Unix seconds; when the player last improved on their own best.
    pub updated_at: u64,
}

#[derive(Debug, Deserialize)]
pub struct Submission {
    pub player_id: String,
    pub name: String,
    pub score: u64,
    #[serde(default)]
    pub wave: u32,
}

#[derive(Debug, Serialize)]
pub struct RankedEntry {
    pub rank: usize,
    pub player_id: String,
    pub name: String,
    pub score: u64,
    pub wave: u32,
    pub updated_at: u64,
}

/// What a submission did to the board.
#[derive(Debug, Serialize)]
pub struct SubmitOutcome {
    /// The player's place on the board after the submission, 1-based.
    pub rank: usize,
    /// The player's best score on record, which is not always the one just sent.
    pub best: u64,
    /// False when the submission was below the player's own best and changed nothing.
    pub improved: bool,
}

#[derive(Debug)]
pub enum SubmitError {
    EmptyName,
    NameTooLong,
    BadPlayerId,
    ScoreTooLarge,
}

impl SubmitError {
    pub fn message(&self) -> &'static str {
        match self {
            SubmitError::EmptyName => "name must not be empty",
            SubmitError::NameTooLong => "name is too long",
            SubmitError::BadPlayerId => "player_id must be 8-64 characters of [A-Za-z0-9_-]",
            SubmitError::ScoreTooLarge => "score is out of range",
        }
    }
}

#[derive(Clone)]
pub struct Leaderboard {
    inner: Arc<Inner>,
}

struct Inner {
    entries: RwLock<HashMap<String, Entry>>,
    updates: broadcast::Sender<String>,
    path: PathBuf,
}

impl Leaderboard {
    /// Loads whatever is on disk. A missing file is an empty board; a corrupt one is
    /// reported and then ignored, because refusing to boot over a bad scoreboard is
    /// worse than starting fresh.
    pub async fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let entries = match tokio::fs::read(&path).await {
            Ok(bytes) => match serde_json::from_slice::<Vec<Entry>>(&bytes) {
                Ok(list) => list
                    .into_iter()
                    .map(|entry| (entry.player_id.clone(), entry))
                    .collect(),
                Err(err) => {
                    eprintln!("leaderboard: {} is not readable ({err}), starting empty", path.display());
                    HashMap::new()
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(err) => {
                eprintln!("leaderboard: could not read {} ({err}), starting empty", path.display());
                HashMap::new()
            }
        };

        let (updates, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            inner: Arc::new(Inner {
                entries: RwLock::new(entries),
                updates,
                path,
            }),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.inner.updates.subscribe()
    }

    pub async fn len(&self) -> usize {
        self.inner.entries.read().await.len()
    }

    /// Top `limit` entries, highest score first. Ties break towards whoever got there
    /// first, so a later equal score cannot displace an earlier one.
    pub async fn top(&self, limit: usize) -> Vec<RankedEntry> {
        let limit = limit.clamp(1, MAX_LIMIT);
        let entries = self.inner.entries.read().await;
        let mut ordered: Vec<&Entry> = entries.values().collect();
        ordered.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then(a.updated_at.cmp(&b.updated_at))
                .then(a.player_id.cmp(&b.player_id))
        });

        ordered
            .into_iter()
            .take(limit)
            .enumerate()
            .map(|(index, entry)| RankedEntry {
                rank: index + 1,
                player_id: entry.player_id.clone(),
                name: entry.name.clone(),
                score: entry.score,
                wave: entry.wave,
                updated_at: entry.updated_at,
            })
            .collect()
    }

    /// The message every client gets on connect and after every change.
    pub async fn snapshot_json(&self, limit: usize) -> String {
        let entries = self.top(limit).await;
        let total = self.len().await;
        serde_json::json!({
            "type": "leaderboard",
            "total": total,
            "entries": entries,
        })
        .to_string()
    }

    /// Records a score, keeping only each player's best. Returns where they now stand.
    pub async fn submit(&self, submission: Submission) -> Result<SubmitOutcome, SubmitError> {
        let name = sanitize_name(&submission.name)?;
        let player_id = validate_player_id(&submission.player_id)?;
        if submission.score > MAX_SCORE {
            return Err(SubmitError::ScoreTooLarge);
        }

        let now = unix_seconds();
        let mut improved = false;
        // A rename changes what everyone sees even when the score does not, so it has
        // to reach disk and the other clients too.
        let mut changed = false;
        {
            let mut entries = self.inner.entries.write().await;
            match entries.get_mut(&player_id) {
                Some(existing) => {
                    // A player renaming themselves should not have to beat their own
                    // score for the new name to show up.
                    if existing.name != name {
                        existing.name = name;
                        changed = true;
                    }
                    if submission.score > existing.score {
                        existing.score = submission.score;
                        existing.wave = submission.wave;
                        existing.updated_at = now;
                        improved = true;
                        changed = true;
                    }
                }
                None => {
                    entries.insert(
                        player_id.clone(),
                        Entry {
                            player_id: player_id.clone(),
                            name,
                            score: submission.score,
                            wave: submission.wave,
                            updated_at: now,
                        },
                    );
                    improved = true;
                    changed = true;
                }
            }
        }

        let (rank, best) = self.standing(&player_id).await;

        if changed {
            self.persist().await;
            let snapshot = self.snapshot_json(DEFAULT_LIMIT).await;
            // Fails only when nobody is listening, which is not an error.
            let _ = self.inner.updates.send(snapshot);
        }

        Ok(SubmitOutcome {
            rank,
            best,
            improved,
        })
    }

    /// A player's rank across the whole board, not just the visible top slice.
    async fn standing(&self, player_id: &str) -> (usize, u64) {
        let entries = self.inner.entries.read().await;
        let Some(entry) = entries.get(player_id) else {
            return (0, 0);
        };

        let ahead = entries
            .values()
            .filter(|other| {
                other.score > entry.score
                    || (other.score == entry.score && other.updated_at < entry.updated_at)
            })
            .count();

        (ahead + 1, entry.score)
    }

    /// Writes to a sibling temp file and renames it over the real one, so a crash
    /// mid-write cannot leave a half-written board behind.
    async fn persist(&self) {
        let entries: Vec<Entry> = {
            let guard = self.inner.entries.read().await;
            guard.values().cloned().collect()
        };

        let json = match serde_json::to_vec_pretty(&entries) {
            Ok(json) => json,
            Err(err) => {
                eprintln!("leaderboard: could not encode board ({err})");
                return;
            }
        };

        let path = &self.inner.path;
        if let Some(parent) = path.parent() {
            if let Err(err) = tokio::fs::create_dir_all(parent).await {
                eprintln!("leaderboard: could not create {} ({err})", parent.display());
                return;
            }
        }

        let temp = path.with_extension("json.tmp");
        if let Err(err) = tokio::fs::write(&temp, &json).await {
            eprintln!("leaderboard: could not write {} ({err})", temp.display());
            return;
        }
        if let Err(err) = tokio::fs::rename(&temp, path).await {
            eprintln!("leaderboard: could not replace {} ({err})", path.display());
        }
    }
}

fn sanitize_name(raw: &str) -> Result<String, SubmitError> {
    // Control characters would let a name break the display it lands in.
    let cleaned: String = raw
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if cleaned.is_empty() {
        return Err(SubmitError::EmptyName);
    }
    if cleaned.chars().count() > MAX_NAME_LEN {
        return Err(SubmitError::NameTooLong);
    }
    Ok(cleaned)
}

fn validate_player_id(raw: &str) -> Result<String, SubmitError> {
    let id = raw.trim();
    let length = id.chars().count();
    let shaped = id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if length < 8 || length > MAX_PLAYER_ID_LEN || !shaped {
        return Err(SubmitError::BadPlayerId);
    }
    Ok(id.to_string())
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
