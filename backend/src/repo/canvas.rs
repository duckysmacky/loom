use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::models::canvas::{
    CanvasArrowResponse, CanvasNoteResponse, CanvasSide, CreateCanvasArrowRequest,
    CreateCanvasNoteRequest, UpdateCanvasNoteRequest,
};

pub const DEFAULT_NOTE_WIDTH: f64 = 200.0;
pub const DEFAULT_NOTE_HEIGHT: f64 = 140.0;

pub async fn list_notes(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<CanvasNoteResponse>, sqlx::Error> {
    sqlx::query_as!(
        CanvasNoteResponse,
        r#"
        SELECT id, body, canvas_x, canvas_y, canvas_width, canvas_height, path_id,
            created_at, updated_at
        FROM canvas_notes
        WHERE user_id = $1
        ORDER BY created_at
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
}

/// `path_id` must already be checked to be one of the caller's paths.
pub async fn create_note(
    user_id: Uuid,
    pool: &PgPool,
    request: &CreateCanvasNoteRequest,
) -> Result<CanvasNoteResponse, sqlx::Error> {
    sqlx::query_as!(
        CanvasNoteResponse,
        r#"
        INSERT INTO canvas_notes
            (user_id, body, canvas_x, canvas_y, canvas_width, canvas_height, path_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, body, canvas_x, canvas_y, canvas_width, canvas_height, path_id,
            created_at, updated_at
        "#,
        user_id,
        request.body,
        request.canvas_x,
        request.canvas_y,
        request.canvas_width.unwrap_or(DEFAULT_NOTE_WIDTH),
        request.canvas_height.unwrap_or(DEFAULT_NOTE_HEIGHT),
        request.path_id,
    )
    .fetch_one(pool)
    .await
}

/// Partial update; `None` fields are left unchanged. A new `path_id` must
/// already be checked to be one of the caller's paths. `None` result means
/// the note doesn't exist or isn't owned by `user_id`.
pub async fn update_note(
    user_id: Uuid,
    pool: &PgPool,
    note_id: Uuid,
    request: &UpdateCanvasNoteRequest,
) -> Result<Option<CanvasNoteResponse>, sqlx::Error> {
    sqlx::query_as!(
        CanvasNoteResponse,
        r#"
        UPDATE canvas_notes SET
            body = COALESCE($3, body),
            canvas_x = COALESCE($4, canvas_x),
            canvas_y = COALESCE($5, canvas_y),
            canvas_width = COALESCE($6, canvas_width),
            canvas_height = COALESCE($7, canvas_height),
            path_id = CASE WHEN $8 THEN $9 ELSE path_id END,
            updated_at = now()
        WHERE id = $2 AND user_id = $1
        RETURNING id, body, canvas_x, canvas_y, canvas_width, canvas_height, path_id,
            created_at, updated_at
        "#,
        user_id,
        note_id,
        request.body,
        request.canvas_x,
        request.canvas_y,
        request.canvas_width,
        request.canvas_height,
        request.path_id.is_some(),
        request.path_id.flatten(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_note(user_id: Uuid, pool: &PgPool, note_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM canvas_notes WHERE id = $2 AND user_id = $1",
        user_id,
        note_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Moves every note inside `path_id` out to the path's own parent (or the
/// top level), keeping where it sits on screen. Used when the path is
/// deleted or stops being a path. ponytail: assumes the path's own position
/// is set; an unplaced path's notes land at their relative offset.
pub async fn release_notes(
    user_id: Uuid,
    connection: &mut PgConnection,
    path_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE canvas_notes note SET
            path_id = (
                SELECT e.to_node_id FROM edges e
                WHERE e.from_node_id = path.id AND e.kind = 'part_of'
            ),
            canvas_x = note.canvas_x + COALESCE(path.canvas_x, 0),
            canvas_y = note.canvas_y + COALESCE(path.canvas_y, 0),
            updated_at = now()
        FROM nodes path
        WHERE note.path_id = $2 AND path.id = $2 AND path.user_id = $1
        "#,
        user_id,
        path_id,
    )
    .execute(connection)
    .await?;
    Ok(())
}

pub async fn list_arrows(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<CanvasArrowResponse>, sqlx::Error> {
    sqlx::query_as!(
        CanvasArrowResponse,
        r#"
        SELECT id, from_node_id, from_note_id, to_node_id, to_note_id,
               from_side AS "from_side: CanvasSide", to_side AS "to_side: CanvasSide", created_at
        FROM canvas_arrows
        WHERE user_id = $1
        ORDER BY created_at
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
}

/// `None` means an end doesn't exist or isn't owned by `user_id` (the
/// INSERT ... SELECT matches no row).
pub async fn create_arrow(
    user_id: Uuid,
    pool: &PgPool,
    request: &CreateCanvasArrowRequest,
) -> Result<Option<CanvasArrowResponse>, sqlx::Error> {
    sqlx::query_as!(
        CanvasArrowResponse,
        r#"
        INSERT INTO canvas_arrows
            (user_id, from_node_id, from_note_id, to_node_id, to_note_id, from_side, to_side)
        SELECT $1, $2, $3, $4, $5, $6, $7
        WHERE ($2::uuid IS NULL OR EXISTS (SELECT 1 FROM nodes WHERE id = $2 AND user_id = $1))
          AND ($3::uuid IS NULL OR EXISTS (SELECT 1 FROM canvas_notes WHERE id = $3 AND user_id = $1))
          AND ($4::uuid IS NULL OR EXISTS (SELECT 1 FROM nodes WHERE id = $4 AND user_id = $1))
          AND ($5::uuid IS NULL OR EXISTS (SELECT 1 FROM canvas_notes WHERE id = $5 AND user_id = $1))
        RETURNING id, from_node_id, from_note_id, to_node_id, to_note_id,
                  from_side AS "from_side: CanvasSide", to_side AS "to_side: CanvasSide", created_at
        "#,
        user_id,
        request.from_node_id,
        request.from_note_id,
        request.to_node_id,
        request.to_note_id,
        request.from_side.unwrap_or(CanvasSide::Right) as CanvasSide,
        request.to_side.unwrap_or(CanvasSide::Left) as CanvasSide,
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_arrow(
    user_id: Uuid,
    pool: &PgPool,
    arrow_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM canvas_arrows WHERE id = $2 AND user_id = $1",
        user_id,
        arrow_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}
