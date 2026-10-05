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
    payload::{
        EditNoteRequest, LessonFilter, NewNoteRequest, validate_new_note,
        validate_note_body,
    },
    refusal::{Refusal, refuse, refuse_all},
    store::{self, StoreError},
    target::{self, NoteTarget},
};
use crate::{
    api::AppState,
    cache::CachePolicy,
    request::{JsonBody, ListQuery, PageSize},
    response::{self, PaginatedResponse, json},
    session::Session,
};

/// A reader's notes, newest first: all of them, or one lesson's.
///
/// Two `Query` extractors, not one struct with five fields: paging and search
/// are `request::ListQuery`'s and every listing has them, while the lesson pair
/// is this endpoint's alone.
pub(crate) async fn list(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(query): Query<ListQuery>,
    Query(filter): Query<LessonFilter>,
) -> Response {
    let paging = query.paging(PageSize::DEFAULT);
    let pattern = query.pattern();
    let pattern = pattern.as_deref();

    let lesson_id = match filter.slugs() {
        Ok(None) => None,
        Ok(Some((book, lesson))) => {
            match store::lesson_id(&state.db, book, lesson).await {
                // A lesson that is gone has no notes to show. Answering with
                // an empty page rather than a 404 keeps a reader on a page
                // whose content the api is still serving.
                Ok(None) => return empty_page(paging),
                Ok(Some(id)) => Some(id),
                Err(error) => {
                    tracing::error!(
                        ?error,
                        book,
                        lesson,
                        "failed to resolve the lesson"
                    );
                    return response::server_error();
                }
            }
        }
        Err(cause_of) => return refuse(cause_of),
    };

    let Ok(notes) =
        store::page(&state.db, session.user_id, pattern, lesson_id, paging)
            .await
    else {
        tracing::error!(user_id = session.user_id, "failed to read notes");
        return response::server_error();
    };

    let Ok(total) =
        store::count(&state.db, session.user_id, pattern, lesson_id).await
    else {
        tracing::error!(user_id = session.user_id, "failed to count notes");
        return response::server_error();
    };

    json(
        StatusCode::OK,
        PaginatedResponse::new(notes, paging, total),
        CachePolicy::NoStore,
    )
}

/// The envelope with nothing in it, in the shape a client already reads.
fn empty_page(paging: crate::request::Paging) -> Response {
    json(
        StatusCode::OK,
        PaginatedResponse::<super::note::Note>::new(Vec::new(), paging, 0),
        CachePolicy::NoStore,
    )
}

pub(crate) async fn create(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    JsonBody(payload): JsonBody<NewNoteRequest>,
) -> Response {
    let checked = match validate_new_note(&payload) {
        Ok(checked) => checked,
        Err(refused) => return refuse_all(&refused),
    };

    let lesson_id =
        match target::resolve(&state.db, &payload, session.user_id).await {
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
                %lesson_id,
                "lesson deleted mid-write"
            );
            response::not_found()
        }
        Err(StoreError::Database(error)) => {
            tracing::error!(
                %error,
                user_id = session.user_id,
                %lesson_id,
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
    JsonBody(payload): JsonBody<EditNoteRequest>,
) -> Response {
    let content = match validate_note_body(&payload.note_content) {
        Ok(content) => content,
        Err(cause_of) => return refuse_all(&[cause_of]),
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
        Ok(true) => {
            response::empty(StatusCode::NO_CONTENT, CachePolicy::NoStore)
        }
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
