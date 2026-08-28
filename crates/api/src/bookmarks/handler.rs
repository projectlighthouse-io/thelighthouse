//! The three handlers, and nothing else.
//!
//! They hand the pool to `store` and nothing else — no statement, no column
//! name, no constraint name. A handler that knew one would have to change when
//! the schema does.

use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};

use uuid::Uuid;

use super::{
    payload::{NewBookmarkRequest, validate},
    refusal::refuse,
    store::{self, StoreError},
};
use crate::{
    api::AppState,
    cache::CachePolicy,
    response::{self, json},
    session::Session,
};

/// What the two path slugs resolved to.
///
/// All three handlers start by asking, and an enum rather than
/// `Result<i64, Response>` for the reason clippy gives: a whole `Response` is
/// 128 bytes to carry through the success path as well. `notes::NoteTarget` is
/// the same shape for the same reason — here it stays beside the handlers
/// because there is no parent to check and nothing else to decide.
enum Lesson {
    Found(Uuid),
    /// No such book or lesson.
    Missing,
    /// The database could not answer. Already logged.
    FailedToQuery,
}

async fn resolve(state: &AppState, book: &str, lesson: &str) -> Lesson {
    match store::lesson_id(&state.db, book, lesson).await {
        Ok(Some(id)) => Lesson::Found(id),
        Ok(None) => Lesson::Missing,
        Err(error) => {
            tracing::error!(
                ?error,
                book,
                lesson,
                "failed to resolve the lesson"
            );
            Lesson::FailedToQuery
        }
    }
}

/// This reader's bookmark in one lesson.
///
/// 404 when there is none, so "no bookmark" and "no such lesson" answer the
/// same. The alternative — 200 with a null body — makes the common case look
/// like a successful read of nothing, and the page has to branch on the body
/// either way.
pub(crate) async fn show(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path((book, lesson)): Path<(String, String)>,
) -> Response {
    let lesson_id = match resolve(&state, &book, &lesson).await {
        Lesson::Found(id) => id,
        Lesson::Missing => return response::not_found(),
        Lesson::FailedToQuery => return response::server_error(),
    };

    match store::find(&state.db, session.user_id, lesson_id).await {
        Ok(Some(bookmark)) => {
            json(StatusCode::OK, bookmark, CachePolicy::NoStore)
        }
        Ok(None) => response::not_found(),
        Err(error) => {
            tracing::error!(
                ?error,
                user_id = session.user_id,
                %lesson_id,
                "failed to read the bookmark"
            );
            response::server_error()
        }
    }
}

/// Places this reader's bookmark, replacing whatever was there.
///
/// `PUT`, not `POST`: there is one bookmark per reader per lesson, so sending
/// the same passage twice leaves the same single row rather than a second one.
pub(crate) async fn place(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path((book, lesson)): Path<(String, String)>,
    axum::Json(payload): axum::Json<NewBookmarkRequest>,
) -> Response {
    let checked = match validate(&payload) {
        Ok(checked) => checked,
        Err(cause_of) => return refuse(cause_of),
    };

    let lesson_id = match resolve(&state, &book, &lesson).await {
        Lesson::Found(id) => id,
        Lesson::Missing => return response::not_found(),
        Lesson::FailedToQuery => return response::server_error(),
    };

    match store::place(&state.db, session.user_id, lesson_id, &checked).await {
        Ok(bookmark) => json(StatusCode::OK, bookmark, CachePolicy::NoStore),
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
                "failed to save the bookmark"
            );
            response::server_error()
        }
    }
}

/// Removes this reader's bookmark. 404 when there was none — see [`show`].
pub(crate) async fn remove(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path((book, lesson)): Path<(String, String)>,
) -> Response {
    let lesson_id = match resolve(&state, &book, &lesson).await {
        Lesson::Found(id) => id,
        Lesson::Missing => return response::not_found(),
        Lesson::FailedToQuery => return response::server_error(),
    };

    match store::clear(&state.db, session.user_id, lesson_id).await {
        Ok(true) => {
            response::empty(StatusCode::NO_CONTENT, CachePolicy::NoStore)
        }
        Ok(false) => response::not_found(),
        Err(error) => {
            tracing::error!(
                ?error,
                user_id = session.user_id,
                %lesson_id,
                "failed to delete the bookmark"
            );
            response::server_error()
        }
    }
}
