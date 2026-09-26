use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::models::active_period::ActivePeriodResponse;
use crate::models::node::NodeStatus;

/// Chronological, oldest first - a newly added period reads as appended at
/// the end. `None` means `node_id` isn't owned by `user_id`; `Some(vec)`
/// (possibly empty) means it is - same explicit ownership check as
/// `checklist::list_items`/`pokes::list_pokes`.
pub async fn list_periods(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
) -> Result<Option<Vec<ActivePeriodResponse>>, sqlx::Error> {
    let owned = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM nodes WHERE id = $2 AND user_id = $1) AS "owned!""#,
        user_id,
        node_id,
    )
    .fetch_one(pool)
    .await?;
    if !owned {
        return Ok(None);
    }

    let periods = sqlx::query_as!(
        ActivePeriodResponse,
        r#"
        SELECT a.id, a.node_id, a.started_at, a.ended_at
        FROM active_periods a
        JOIN nodes n ON n.id = a.node_id
        WHERE n.user_id = $1 AND a.node_id = $2
        ORDER BY a.started_at, a.id
        "#,
        user_id,
        node_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(Some(periods))
}

/// `None` means the node isn't owned by `user_id` (the INSERT ... SELECT
/// matches no row).
pub async fn create_period(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
) -> Result<Option<ActivePeriodResponse>, sqlx::Error> {
    sqlx::query_as!(
        ActivePeriodResponse,
        r#"
        INSERT INTO active_periods (node_id, started_at, ended_at)
        SELECT n.id, $3, $4
        FROM nodes n
        WHERE n.id = $2 AND n.user_id = $1
        RETURNING id, node_id, started_at, ended_at
        "#,
        user_id,
        node_id,
        started_at,
        ended_at,
    )
    .fetch_optional(pool)
    .await
}

/// Partial update; omitted fields are left unchanged. `ended_at_set` follows
/// the double-option PATCH idiom: `true` + `None` clears it back to open.
pub async fn update_period(
    user_id: Uuid,
    pool: &PgPool,
    period_id: Uuid,
    started_at: Option<DateTime<Utc>>,
    ended_at_set: bool,
    ended_at: Option<DateTime<Utc>>,
) -> Result<Option<ActivePeriodResponse>, sqlx::Error> {
    sqlx::query_as!(
        ActivePeriodResponse,
        r#"
        UPDATE active_periods a SET
            started_at = COALESCE($3, a.started_at),
            ended_at = CASE WHEN $4 THEN $5 ELSE a.ended_at END
        FROM nodes n
        WHERE a.id = $2 AND n.id = a.node_id AND n.user_id = $1
        RETURNING a.id, a.node_id, a.started_at, a.ended_at
        "#,
        user_id,
        period_id,
        started_at,
        ended_at_set,
        ended_at,
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_period(
    user_id: Uuid,
    pool: &PgPool,
    period_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM active_periods a
        USING nodes n
        WHERE a.id = $2 AND n.id = a.node_id AND n.user_id = $1
        "#,
        user_id,
        period_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// The node's latest period (its end is the node's completed date while
/// done) - `None` when the node has never started.
async fn last_period(
    user_id: Uuid,
    tx: &mut PgConnection,
    node_id: Uuid,
) -> Result<Option<(Uuid, Option<DateTime<Utc>>)>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT a.id, a.ended_at
        FROM active_periods a
        JOIN nodes n ON n.id = a.node_id
        WHERE a.node_id = $2 AND n.user_id = $1
        ORDER BY a.started_at DESC, a.id DESC
        LIMIT 1
        "#,
        user_id,
        node_id,
    )
    .fetch_optional(&mut *tx)
    .await?;
    Ok(row.map(|row| (row.id, row.ended_at)))
}

async fn open_period(
    user_id: Uuid,
    tx: &mut PgConnection,
    node_id: Uuid,
    closed: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO active_periods (node_id, started_at, ended_at)
        SELECT n.id, now(), CASE WHEN $3 THEN now() END FROM nodes n
        WHERE n.id = $2 AND n.user_id = $1
        "#,
        user_id,
        node_id,
        closed,
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// Sets a period's end; `None` reopens it.
async fn set_period_end(
    user_id: Uuid,
    tx: &mut PgConnection,
    period_id: Uuid,
    ended_at: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE active_periods a SET ended_at = $3
        FROM nodes n
        WHERE a.id = $2 AND n.id = a.node_id AND n.user_id = $1
        "#,
        user_id,
        period_id,
        ended_at,
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// Closes a period now - clamped to its start, so a hand-set future start
/// never makes the close violate the `ended_at >= started_at` check.
async fn close_period_now(
    user_id: Uuid,
    tx: &mut PgConnection,
    period_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE active_periods a SET ended_at = GREATEST(now(), a.started_at)
        FROM nodes n
        WHERE a.id = $2 AND n.id = a.node_id AND n.user_id = $1
        "#,
        user_id,
        period_id,
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// Keeps the periods in step with a status change, inside the caller's
/// transaction. A node's started date is its first period's start and its
/// completed date is the last period's end while done, so this is what
/// moves those edges:
///
/// - `-> active`: the first activation opens the first period. After a
///   closed period, `track_active_periods` opens a new one; with tracking
///   off the last period is reopened instead (one continuous span).
/// - `-> done`: closes the open period - its end becomes the completed
///   date. If the last period already closed (a pause), a completion-day
///   period is added so the pause record isn't overwritten. A node that
///   never started gets nothing: no start means no completed date either.
/// - `active -> paused/archived` with tracking on closes the open period.
/// - `queued`/`idea` are neutral.
pub async fn sync_status_transition(
    user_id: Uuid,
    tx: &mut PgConnection,
    node_id: Uuid,
    old_status: NodeStatus,
    new_status: NodeStatus,
    track_active_periods: bool,
) -> Result<(), sqlx::Error> {
    if old_status == new_status {
        return Ok(());
    }
    let last = last_period(user_id, tx, node_id).await?;
    match (new_status, last) {
        (NodeStatus::Active, None) => open_period(user_id, tx, node_id, false).await?,
        (NodeStatus::Active, Some((period_id, Some(_)))) => {
            if track_active_periods {
                open_period(user_id, tx, node_id, false).await?;
            } else {
                set_period_end(user_id, tx, period_id, None).await?;
            }
        }
        (NodeStatus::Done, Some((period_id, None))) => {
            close_period_now(user_id, tx, period_id).await?
        }
        (NodeStatus::Done, Some((_, Some(_)))) => open_period(user_id, tx, node_id, true).await?,
        (NodeStatus::Paused | NodeStatus::Archived, Some((period_id, None)))
            if old_status == NodeStatus::Active && track_active_periods =>
        {
            close_period_now(user_id, tx, period_id).await?
        }
        _ => {}
    }
    Ok(())
}

/// An explicit started date moves the first period's start; with no periods
/// it opens one there. `None` (clearing it) removes every period - the node
/// is back to "not started".
pub async fn set_started(
    user_id: Uuid,
    tx: &mut PgConnection,
    node_id: Uuid,
    started_at: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    let Some(started_at) = started_at else {
        sqlx::query!(
            r#"
            DELETE FROM active_periods a
            USING nodes n
            WHERE a.node_id = $2 AND n.id = a.node_id AND n.user_id = $1
            "#,
            user_id,
            node_id,
        )
        .execute(&mut *tx)
        .await?;
        return Ok(());
    };

    let moved = sqlx::query!(
        r#"
        UPDATE active_periods a SET started_at = $3
        FROM nodes n
        WHERE n.id = a.node_id AND n.user_id = $1
          AND a.id = (
              SELECT id FROM active_periods WHERE node_id = $2
              ORDER BY started_at, id LIMIT 1
          )
        "#,
        user_id,
        node_id,
        started_at,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if moved == 0 {
        sqlx::query!(
            r#"
            INSERT INTO active_periods (node_id, started_at)
            SELECT n.id, $3 FROM nodes n WHERE n.id = $2 AND n.user_id = $1
            "#,
            user_id,
            node_id,
            started_at,
        )
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

/// An explicit completed date moves the last period's end; `None` reopens
/// it. No-op for a node without periods (the handler rejects a completed
/// date on a node that never started).
pub async fn set_completed(
    user_id: Uuid,
    tx: &mut PgConnection,
    node_id: Uuid,
    completed_at: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    if let Some((period_id, _)) = last_period(user_id, tx, node_id).await? {
        set_period_end(user_id, tx, period_id, completed_at).await?;
    }
    Ok(())
}
