use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::models::active_period::ActivePeriodResponse;
use crate::models::node::NodeStatus;

/// `None` means `node_id` isn't owned by `user_id`; `Some(vec)` (possibly
/// empty) means it is - same explicit ownership check as
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
        ORDER BY a.started_at DESC
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

/// Applies the open/close rule for one status transition, inside the
/// caller's transaction - called from `nodes::update_node` only when the
/// client opted in via `track_active_periods`. No-op unless the transition
/// actually crosses into or out of `active`; never touches
/// `nodes.started_at`/`completed_at`, which stamp themselves regardless.
///
/// - Leaving `active` for `paused` or `archived` closes the open period.
///   If none exists yet, materialize it immediately from the node's own
///   `started_at` - there's no other durable timestamp for "when this
///   happened" the way `done` has `completed_at`.
/// - Leaving `active` for `done` only closes an *already-open* period (from
///   an earlier pause/resume cycle); with zero periods it does nothing,
///   relying on `completed_at` alone - that's the "finished in one
///   continuous run" case, which should stay at zero periods.
/// - Entering `active` from anything else opens a new period, but only on
///   reactivation (`old_started_at` already set). If no periods exist yet
///   and the node has an existing `completed_at` (a single-run `done` node
///   being reactivated), first migrate that started_at/completed_at span
///   into period #1 - the "migrate the first period" case.
/// - `queued`/`idea` are neutral in both directions.
pub async fn apply_status_transition(
    tx: &mut PgConnection,
    node_id: Uuid,
    old_status: NodeStatus,
    new_status: NodeStatus,
    old_started_at: Option<DateTime<Utc>>,
    old_completed_at: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    let was_active = old_status == NodeStatus::Active;
    let opens = new_status == NodeStatus::Active && !was_active && old_started_at.is_some();
    let closes = was_active
        && matches!(
            new_status,
            NodeStatus::Paused | NodeStatus::Done | NodeStatus::Archived
        );

    if closes {
        let closed_one = sqlx::query!(
            "UPDATE active_periods SET ended_at = now() WHERE node_id = $1 AND ended_at IS NULL",
            node_id,
        )
        .execute(&mut *tx)
        .await?
        .rows_affected()
            > 0;

        if !closed_one && new_status != NodeStatus::Done {
            sqlx::query!(
                "INSERT INTO active_periods (node_id, started_at, ended_at)
                 SELECT id, started_at, now() FROM nodes WHERE id = $1 AND started_at IS NOT NULL",
                node_id,
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    if opens {
        let has_any = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM active_periods WHERE node_id = $1) AS "exists!""#,
            node_id,
        )
        .fetch_one(&mut *tx)
        .await?;

        // `opens` already guarantees `old_started_at.is_some()`.
        if !has_any && let Some(completed_at) = old_completed_at {
            sqlx::query!(
                "INSERT INTO active_periods (node_id, started_at, ended_at) VALUES ($1, $2, $3)",
                node_id,
                old_started_at.unwrap(),
                completed_at,
            )
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query!(
            "INSERT INTO active_periods (node_id, started_at) VALUES ($1, now())",
            node_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    Ok(())
}
