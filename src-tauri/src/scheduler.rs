use rusqlite::{params, Connection};
use serde_json::json;
use uuid::Uuid;

use crate::{error::AppError, graph};

#[taurpc::ipc_type]
pub struct SchedulerDescriptor {
    pub key: String,
    pub name: String,
    pub description: String,
}

#[taurpc::ipc_type]
pub struct ReviewEvent {
    pub id: String,
    pub node_id: String,
    pub grade: String,
    pub scheduler_key: String,
    pub reviewed_at: String,
    pub previous_status: String,
    pub next_status: String,
    pub scheduled_for_at: Option<String>,
    pub scheduler_state: String,
}

#[derive(Clone)]
struct ProgressSnapshot {
    status: String,
    review_count: i32,
    streak: i32,
    scheduler_key: String,
}

#[derive(Clone, Copy)]
enum ReviewGrade {
    Again,
    Hard,
    Good,
    Easy,
}

struct ScheduleDecision {
    status: String,
    review_count_delta: i32,
    streak: i32,
    next_review_at: Option<String>,
    scheduler_key: String,
    scheduler_state: String,
}

trait SchedulerAlgorithm {
    fn key(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn review(&self, now_ts: i64, current: &ProgressSnapshot, grade: ReviewGrade) -> ScheduleDecision;
}

struct BasicScheduler;
struct CramScheduler;

impl SchedulerAlgorithm for BasicScheduler {
    fn key(&self) -> &'static str {
        "basic-v1"
    }

    fn name(&self) -> &'static str {
        "Basic Scheduler"
    }

    fn description(&self) -> &'static str {
        "Simple built-in spaced repetition scheduler using Again/Hard/Good/Easy grades."
    }

    fn review(&self, now_ts: i64, current: &ProgressSnapshot, grade: ReviewGrade) -> ScheduleDecision {
        let current_streak = current.streak.max(0);

        let (status, next_in_seconds, streak) = match grade {
            ReviewGrade::Again => ("learning".to_string(), Some(0_i64), 0),
            ReviewGrade::Hard => {
                let interval = if current.review_count == 0 { 6 * 60 * 60 } else { 24 * 60 * 60 };
                let next_status = if current.status == "new" { "learning" } else { "review" };
                (next_status.to_string(), Some(interval), current_streak)
            }
            ReviewGrade::Good => {
                let next_streak = current_streak + 1;
                let interval = match next_streak {
                    0 | 1 => 24 * 60 * 60,
                    2 => 3 * 24 * 60 * 60,
                    3 => 7 * 24 * 60 * 60,
                    4 => 14 * 24 * 60 * 60,
                    _ => 30 * 24 * 60 * 60,
                };
                let next_status = if next_streak >= 4 { "mastered" } else { "review" };
                (next_status.to_string(), Some(interval), next_streak)
            }
            ReviewGrade::Easy => {
                let next_streak = current_streak + 2;
                let interval = match next_streak {
                    0..=2 => 7 * 24 * 60 * 60,
                    3..=4 => 21 * 24 * 60 * 60,
                    _ => 45 * 24 * 60 * 60,
                };
                ("mastered".to_string(), Some(interval), next_streak)
            }
        };

        let next_review_at = next_in_seconds.map(|interval| (now_ts + interval).to_string());
        let scheduler_state = json!({
            "interval_seconds": next_in_seconds.unwrap_or(0),
            "streak": streak,
            "last_grade": grade_label(grade),
        })
        .to_string();

        ScheduleDecision {
            status,
            review_count_delta: 1,
            streak,
            next_review_at,
            scheduler_key: self.key().to_string(),
            scheduler_state,
        }
    }
}

impl SchedulerAlgorithm for CramScheduler {
    fn key(&self) -> &'static str {
        "cram-v1"
    }

    fn name(&self) -> &'static str {
        "Cram Scheduler"
    }

    fn description(&self) -> &'static str {
        "Short-interval scheduler for intensive temporary review sessions."
    }

    fn review(&self, now_ts: i64, current: &ProgressSnapshot, grade: ReviewGrade) -> ScheduleDecision {
        let current_streak = current.streak.max(0);
        let (status, next_in_seconds, streak) = match grade {
            ReviewGrade::Again => ("learning".to_string(), Some(5 * 60), 0),
            ReviewGrade::Hard => ("learning".to_string(), Some(20 * 60), current_streak),
            ReviewGrade::Good => ("review".to_string(), Some(60 * 60), current_streak + 1),
            ReviewGrade::Easy => ("mastered".to_string(), Some(6 * 60 * 60), current_streak + 1),
        };

        let next_review_at = next_in_seconds.map(|interval| (now_ts + interval).to_string());
        let scheduler_state = json!({
            "interval_seconds": next_in_seconds.unwrap_or(0),
            "streak": streak,
            "last_grade": grade_label(grade),
        })
        .to_string();

        ScheduleDecision {
            status,
            review_count_delta: 1,
            streak,
            next_review_at,
            scheduler_key: self.key().to_string(),
            scheduler_state,
        }
    }
}

fn grade_label(grade: ReviewGrade) -> &'static str {
    match grade {
        ReviewGrade::Again => "again",
        ReviewGrade::Hard => "hard",
        ReviewGrade::Good => "good",
        ReviewGrade::Easy => "easy",
    }
}

fn now_ts_i64() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn available_algorithms() -> Vec<Box<dyn SchedulerAlgorithm>> {
    vec![Box::new(BasicScheduler), Box::new(CramScheduler)]
}

fn default_algorithm_key() -> &'static str {
    "basic-v1"
}

fn resolve_algorithm(key: Option<&str>) -> Result<Box<dyn SchedulerAlgorithm>, AppError> {
    let requested = key.unwrap_or(default_algorithm_key());
    available_algorithms()
        .into_iter()
        .find(|algorithm| algorithm.key() == requested)
        .ok_or_else(|| AppError::Other(format!("Unknown scheduler algorithm: {}", requested)))
}

fn parse_grade(grade: &str) -> Result<ReviewGrade, AppError> {
    match grade.trim().to_ascii_lowercase().as_str() {
        "again" => Ok(ReviewGrade::Again),
        "hard" => Ok(ReviewGrade::Hard),
        "good" => Ok(ReviewGrade::Good),
        "easy" => Ok(ReviewGrade::Easy),
        other => Err(AppError::Other(format!("Unknown review grade: {}", other))),
    }
}

fn load_progress_snapshot(conn: &Connection, node_id: &str) -> Result<ProgressSnapshot, AppError> {
    let row = conn.query_row(
        "SELECT COALESCE(status, 'new'),
                COALESCE(review_count, 0),
                COALESCE(streak, 0),
                COALESCE(scheduler_key, 'basic-v1')
         FROM node_progress
         WHERE node_id = ?1",
        [node_id],
        |row| {
            Ok(ProgressSnapshot {
                status: row.get(0)?,
                review_count: row.get(1)?,
                streak: row.get(2)?,
                scheduler_key: row.get(3)?,
            })
        },
    );

    match row {
        Ok(snapshot) => Ok(snapshot),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(ProgressSnapshot {
            status: "new".to_string(),
            review_count: 0,
            streak: 0,
            scheduler_key: default_algorithm_key().to_string(),
        }),
        Err(other) => Err(AppError::Database(other)),
    }
}

pub fn query_scheduler_descriptors() -> Vec<SchedulerDescriptor> {
    available_algorithms()
        .into_iter()
        .map(|algorithm| SchedulerDescriptor {
            key: algorithm.key().to_string(),
            name: algorithm.name().to_string(),
            description: algorithm.description().to_string(),
        })
        .collect()
}

pub fn query_review_events(conn: &Connection) -> Result<Vec<ReviewEvent>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, node_id, grade, scheduler_key, reviewed_at, previous_status, next_status, scheduled_for_at, scheduler_state
         FROM review_events
         ORDER BY reviewed_at DESC, id DESC
         LIMIT 120",
    )?;
    let events = stmt
        .query_map([], |row| {
            Ok(ReviewEvent {
                id: row.get(0)?,
                node_id: row.get(1)?,
                grade: row.get(2)?,
                scheduler_key: row.get(3)?,
                reviewed_at: row.get(4)?,
                previous_status: row.get(5)?,
                next_status: row.get(6)?,
                scheduled_for_at: row.get(7)?,
                scheduler_state: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(events)
}

pub fn review_node(
    conn: &Connection,
    node_id: &str,
    grade: &str,
    scheduler_key: Option<&str>,
) -> Result<graph::Node, AppError> {
    let _ = graph::query_single_node(conn, node_id)?;

    let now_ts = now_ts_i64();
    let current = load_progress_snapshot(conn, node_id)?;
    let parsed_grade = parse_grade(grade)?;
    let algorithm = resolve_algorithm(scheduler_key.or(Some(current.scheduler_key.as_str())))?;
    let decision = algorithm.review(now_ts, &current, parsed_grade);
    let updated_at = now_ts.to_string();

    conn.execute(
        "INSERT INTO node_progress
            (node_id, status, review_count, streak, last_reviewed_at, next_review_at, scheduler_key, scheduler_state, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?5, ?5)
         ON CONFLICT(node_id) DO UPDATE SET
            status = excluded.status,
            review_count = node_progress.review_count + excluded.review_count,
            streak = excluded.streak,
            last_reviewed_at = excluded.last_reviewed_at,
            next_review_at = excluded.next_review_at,
            scheduler_key = excluded.scheduler_key,
            scheduler_state = excluded.scheduler_state,
            updated_at = excluded.updated_at",
        params![
            node_id,
            decision.status,
            decision.review_count_delta,
            decision.streak,
            updated_at,
            decision.next_review_at,
            decision.scheduler_key,
            decision.scheduler_state
        ],
    )?;

    let learned = if decision.status == "mastered" { 1 } else { 0 };
    conn.execute(
        "UPDATE nodes SET learned = ?1 WHERE id = ?2",
        params![learned, node_id],
    )?;

    conn.execute(
        "INSERT INTO review_events
            (id, node_id, grade, scheduler_key, reviewed_at, previous_status, next_status, scheduled_for_at, scheduler_state)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            Uuid::new_v4().to_string(),
            node_id,
            grade_label(parsed_grade),
            algorithm.key(),
            updated_at,
            current.status,
            decision.status,
            decision.next_review_at,
            decision.scheduler_state
        ],
    )?;

    graph::query_single_node(conn, node_id)
}
