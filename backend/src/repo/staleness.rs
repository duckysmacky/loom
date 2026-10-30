use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::node::{NodeKind, NodeResponse, NodeStatus};
use crate::models::settings::{PathStaleMode, Settings, StaleMode, StalePeriodEnd};
use crate::repo::{active_periods, settings};

/// Brings every node's `stale_since` in line with the account's settings:
/// stamps nodes that went untouched past the threshold (at the moment they
/// crossed it) and clears ones touched again or no longer eligible. A node
/// stays stale through a status change - only a fresh poke, done, archived
/// or an excluded kind clears it - so auto-pause fires once, on entry.
/// Paths go after the nodes inside them, per `path_stale_mode`.
pub async fn sweep(user_id: Uuid, pool: &PgPool, settings: &Settings) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let changed = sqlx::query!(
        r#"
        WITH leaf AS (
            SELECT n.id, n.kind, n.status, n.archived, n.stale_since,
                   MAX(p.poked_at) AS last_poked_at,
                   COALESCE(MAX(p.poked_at), n.created_at) + make_interval(days => $2) AS stale_at
            FROM nodes n LEFT JOIN pokes p ON p.node_id = n.id
            WHERE n.user_id = $1 AND n.kind <> 'path'
            GROUP BY n.id
        ),
        target AS (
            SELECT id, stale_since AS old_since, last_poked_at,
                CASE WHEN NOT archived AND status <> 'done' AND stale_at < now()
                    AND ((kind = 'project' AND $3) OR (kind = 'study' AND $4))
                    AND (stale_since IS NOT NULL
                         OR status = 'active'
                         OR (status = 'queued' AND stale_mode_allows($5, last_poked_at IS NOT NULL))
                         OR (status = 'paused' AND stale_mode_allows($6, last_poked_at IS NOT NULL))
                         OR (status = 'idea' AND stale_mode_allows($7, last_poked_at IS NOT NULL)))
                THEN COALESCE(stale_since, stale_at) END AS new_since
            FROM leaf
        )
        UPDATE nodes n SET stale_since = t.new_since
        FROM target t
        WHERE n.id = t.id AND n.stale_since IS DISTINCT FROM t.new_since
        RETURNING n.id, n.status AS "status?: NodeStatus", t.old_since, t.new_since,
                  t.last_poked_at
        "#,
        user_id,
        settings.stale_days as i32,
        settings.stale_kinds.contains(&NodeKind::Project),
        settings.stale_kinds.contains(&NodeKind::Study),
        mode_name(settings.stale_queued),
        mode_name(settings.stale_paused),
        mode_name(settings.stale_idea_status),
    )
    .fetch_all(&mut *tx)
    .await?;

    if settings.stale_auto_pause {
        for node in changed {
            let (None, Some(stale_since)) = (node.old_since, node.new_since) else {
                continue;
            };
            if node.status != Some(NodeStatus::Active) {
                continue;
            }
            sqlx::query!(
                "UPDATE nodes SET status = 'paused', updated_at = now() WHERE id = $2 AND user_id = $1",
                user_id,
                node.id,
            )
            .execute(&mut *tx)
            .await?;
            if settings.track_active_periods {
                let ended_at = match settings.stale_pause_period_end {
                    StalePeriodEnd::LastPoke => node.last_poked_at,
                    StalePeriodEnd::StaleMoment => Some(stale_since),
                };
                active_periods::close_open_period_at(user_id, &mut tx, node.id, ended_at).await?;
            }
        }
    }

    // A path in `all` mode counts the nodes still in play inside it: done
    // ones are ignored, and so are idea/queued/paused ones that aren't stale.
    sqlx::query!(
        r#"
        WITH target AS (
            SELECT r.id,
                CASE
                    WHEN r.archived OR r.status = 'done' THEN NULL
                    WHEN $3 = 'all' THEN (
                        SELECT CASE WHEN bool_and(leaf.stale_since IS NOT NULL)
                                    THEN MAX(leaf.stale_since) END
                        FROM path_leaves(r.id) AS inside(leaf_id)
                        JOIN nodes leaf ON leaf.id = inside.leaf_id
                        WHERE NOT leaf.archived AND leaf.status <> 'done'
                          AND (leaf.stale_since IS NOT NULL OR leaf.status = 'active')
                    )
                    WHEN $3 = 'any'
                        AND COALESCE(r.last_poked_at, r.created_at) + make_interval(days => $2) < now()
                        AND (r.stale_since IS NOT NULL
                             OR r.status = 'active'
                             OR (r.status = 'queued' AND stale_mode_allows($4, r.last_poked_at IS NOT NULL))
                             OR (r.status = 'paused' AND stale_mode_allows($5, r.last_poked_at IS NOT NULL))
                             OR (r.status = 'idea' AND stale_mode_allows($6, r.last_poked_at IS NOT NULL)))
                    THEN COALESCE(r.stale_since,
                                  COALESCE(r.last_poked_at, r.created_at) + make_interval(days => $2))
                END AS new_since
            FROM node_rows r
            WHERE r.user_id = $1 AND r.kind = 'path'
        )
        UPDATE nodes n SET stale_since = t.new_since
        FROM target t
        WHERE n.id = t.id AND n.stale_since IS DISTINCT FROM t.new_since
        "#,
        user_id,
        settings.stale_days as i32,
        path_mode_name(settings.path_stale_mode),
        mode_name(settings.stale_queued),
        mode_name(settings.stale_paused),
        mode_name(settings.stale_idea_status),
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await
}

/// Sweeps every account - the periodic pass that catches nodes crossing
/// the threshold while nobody touches anything. One account failing
/// doesn't stop the rest.
pub async fn sweep_all(pool: &PgPool) -> Result<(), sqlx::Error> {
    let user_ids = sqlx::query_scalar!("SELECT id FROM users")
        .fetch_all(pool)
        .await?;
    for user_id in user_ids {
        let result = match settings::get(user_id, pool).await {
            Ok(settings) => sweep(user_id, pool, &settings).await,
            Err(error) => Err(error),
        };
        if let Err(error) = result {
            tracing::warn!(%user_id, %error, "stale sweep failed");
        }
    }
    Ok(())
}

/// Whether a node that isn't stale yet can go stale - the Rust twin of the
/// sweep's eligibility, for the dashboard's "going stale" warning. Paths
/// only in `any` mode: in `all` mode there's no single date to warn about.
pub fn can_go_stale(node: &NodeResponse, settings: &Settings) -> bool {
    if node.archived {
        return false;
    }
    let kind_allowed = match node.kind {
        NodeKind::Path => settings.path_stale_mode == PathStaleMode::Any,
        NodeKind::Idea => false,
        kind => settings.stale_kinds.contains(&kind),
    };
    let poked = node.last_poked_at.is_some();
    kind_allowed
        && match node.status {
            Some(NodeStatus::Active) => true,
            Some(NodeStatus::Queued) => mode_allows(settings.stale_queued, poked),
            Some(NodeStatus::Paused) => mode_allows(settings.stale_paused, poked),
            Some(NodeStatus::Idea) => mode_allows(settings.stale_idea_status, poked),
            Some(NodeStatus::Done) | None => false,
        }
}

/// When a node goes (or went) stale if left untouched.
pub fn stale_at(node: &NodeResponse, settings: &Settings) -> DateTime<Utc> {
    node.last_poked_at.unwrap_or(node.created_at) + Duration::days(settings.stale_days)
}

fn mode_allows(mode: StaleMode, poked: bool) -> bool {
    match mode {
        StaleMode::Always => true,
        StaleMode::Poked => poked,
        StaleMode::Never => false,
    }
}

fn mode_name(mode: StaleMode) -> &'static str {
    match mode {
        StaleMode::Always => "always",
        StaleMode::Poked => "poked",
        StaleMode::Never => "never",
    }
}

fn path_mode_name(mode: PathStaleMode) -> &'static str {
    match mode {
        PathStaleMode::Off => "off",
        PathStaleMode::All => "all",
        PathStaleMode::Any => "any",
    }
}
