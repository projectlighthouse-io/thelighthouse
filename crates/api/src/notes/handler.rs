//! The four handlers, and nothing else.
//!
//! They hand the pool to `store` and nothing else — no statement, no column
//! name, no constraint name. A handler that knew one would have to change when
//! the schema does.

use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
};

use super::{
    payload::{EditNoteRequest, NewNoteRequest, validate_new_note, validate_note_body},
    refusal::{Refusal, refuse},
    store::{self, StoreError},
    target::{self, NoteTarget},
};
use crate::{
    api::AppState,
    cache::CachePolicy,
    request::{ListQuery, PageSize},
    response::{self, PaginatedResponse, json},
    session::Session,
};

pub(crate) async fn list(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(query): Query<ListQuery>,
) -> Response {
    let paging = query.paging(PageSize::DEFAULT);
    let pattern = query.pattern();
    let pattern = pattern.as_deref();

    let Ok(notes) = store::page(&state.db, session.user_id, pattern, paging).await else {
        tracing::error!(user_id = session.user_id, "failed to read notes");
        return response::server_error();
    };

    let Ok(total) = store::count(&state.db, session.user_id, pattern).await else {
        tracing::error!(user_id = session.user_id, "failed to count notes");
        return response::server_error();
    };

    json(
        StatusCode::OK,
        PaginatedResponse::new(notes, paging, total),
        CachePolicy::NoStore,
    )
}

pub(crate) async fn create(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    axum::Json(payload): axum::Json<NewNoteRequest>,
) -> Response {
    let checked = match validate_new_note(&payload) {
        Ok(checked) => checked,
        Err(cause_of) => return refuse(cause_of),
    };

    let lesson_id = match target::resolve(&state.db, &payload).await {
        NoteTarget::LessonId(id) => id,
        NoteTarget::Missing => return response::not_found(),
        NoteTarget::Refused(cause_of) => return refuse(cause_of),
        NoteTarget::Failed => return response::server_error(),
    };

    let saved = store::insert(
        &state.db,
        session.user_id,
        lesson_id,
        &checked,
        payload.is_public.unwrap_or(true),
        payload.parent_id,
    )
    .await;

    match saved {
        Ok(note) => json(StatusCode::CREATED, note, CachePolicy::NoStore),
        Err(StoreError::ParentGone) => {
            tracing::info!(
                user_id = session.user_id,
                "replied to a note that was deleted"
            );
            refuse(Refusal::NoSuchNote)
        }
        Err(StoreError::LessonGone) => {
            tracing::info!(
                user_id = session.user_id,
                lesson_id,
                "lesson deleted mid-write"
            );
            response::not_found()
        }
        Err(StoreError::Database(error)) => {
            tracing::error!(
                %error,
                user_id = session.user_id,
                lesson_id,
                "failed to save the note"
            );
            response::server_error()
        }
    }
}

pub(crate) async fn update(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(id): Path<i64>,
    axum::Json(payload): axum::Json<EditNoteRequest>,
) -> Response {
    let content = match validate_note_body(&payload.note_content) {
        Ok(content) => content,
        Err(cause_of) => return refuse(cause_of),
    };

    match store::rewrite(&state.db, id, session.user_id, content).await {
        // The row as stored, not what was sent.
        Ok(Some(note)) => json(StatusCode::OK, note, CachePolicy::NoStore),
        Ok(None) => response::not_found(),
        Err(error) => {
            tracing::error!(
                ?error,
                note_id = id,
                user_id = session.user_id,
                "failed to update the note"
            );
            response::server_error()
        }
    }
}

pub(crate) async fn remove(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(id): Path<i64>,
) -> Response {
    match store::delete(&state.db, id, session.user_id).await {
        Ok(true) => response::empty(StatusCode::NO_CONTENT, CachePolicy::NoStore),
        Ok(false) => response::not_found(),
        Err(error) => {
            tracing::error!(
                ?error,
                note_id = id,
                user_id = session.user_id,
                "failed to delete the note"
            );
            response::server_error()
        }
    }
}
