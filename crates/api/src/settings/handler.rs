//! One function per route. No SQL, and no rendering decisions.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};

use super::{
    refusal::{Refusal, refuse},
    view::{
        EditProfile, MAX_NAME, MintedView, NewToken, ProfileView, TokenView,
    },
};
use crate::{
    api::AppState,
    cache::CachePolicy,
    response::{self, not_found},
    session::Session,
    tokens::{store, token},
};

/// `GET /api/settings/tokens` — every token this reader holds.
pub(crate) async fn list(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    match store::list(&state.db, session.user_id).await {
        Ok(records) => {
            let tokens: Vec<TokenView> =
                records.into_iter().map(TokenView::from).collect();

            response::json(StatusCode::OK, tokens, CachePolicy::NoStore)
        }
        Err(cause) => {
            tracing::error!(%cause, user_id = session.user_id, "failed to list tokens");
            response::server_error()
        }
    }
}

/// `POST /api/settings/tokens` — mint one, and show it once.
///
/// The response is the only time the secret exists outside the reader's
/// clipboard: it is hashed on the way into the row and the plaintext is
/// dropped with the request.
pub(crate) async fn create(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(body): Json<NewToken>,
) -> Response {
    let name = body.name();

    if name.is_empty() {
        return refuse(Refusal::NameRequired);
    }
    // Counted in characters, not bytes: a name of sixty accented letters is
    // sixty characters and should not be refused for being a hundred bytes.
    if name.chars().count() > MAX_NAME {
        return refuse(Refusal::NameTooLong);
    }

    let Some(minted) = token::mint() else {
        tracing::error!("failed to read the system random source");
        return refuse(Refusal::NoRandomness);
    };

    match store::create(&state.db, session.user_id, name, &minted.hash).await {
        Ok(record) => {
            tracing::info!(
                user_id = session.user_id,
                id = record.id,
                "token created"
            );

            let token_string = format!("{}|{}", record.id, minted.secret);

            response::json(
                StatusCode::CREATED,
                MintedView {
                    token: TokenView::from(record),
                    token_string,
                },
                CachePolicy::NoStore,
            )
        }
        Err(cause) => {
            tracing::error!(%cause, user_id = session.user_id, "failed to create a token");
            response::server_error()
        }
    }
}

/// `DELETE /api/settings/tokens/{id}` — revoke one.
///
/// 404 for a token that is not this reader's, which is the same answer as one
/// that does not exist — see `store::revoke`.
pub(crate) async fn revoke(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(id): Path<i64>,
) -> Response {
    match store::revoke(&state.db, session.user_id, id).await {
        Ok(true) => {
            tracing::info!(user_id = session.user_id, id, "token revoked");
            response::empty(StatusCode::NO_CONTENT, CachePolicy::NoStore)
        }
        Ok(false) => not_found(),
        Err(cause) => {
            tracing::error!(%cause, user_id = session.user_id, id, "failed to revoke a token");
            response::server_error()
        }
    }
}

// ------------------------------------------------------------------ profile

/// `GET /api/settings/profile` — what the public profile page renders.
///
/// Name, email and avatar are not here: they come from the provider and the
/// page already has them from `GET /api/auth/session`. This is the part a
/// reader writes.
pub(crate) async fn profile(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Response {
    match crate::users::profile(&state.db, session.user_id).await {
        // A live session pointing at a reader who is gone. The session should
        // have gone with them; until something sweeps it, 404 is the honest
        // answer — the same one `projects::me` gives.
        Ok(None) => not_found(),
        Ok(Some(row)) => response::json(
            StatusCode::OK,
            ProfileView::from(row),
            CachePolicy::NoStore,
        ),
        Err(cause) => {
            tracing::error!(%cause, user_id = session.user_id, "failed to read a profile");
            response::server_error()
        }
    }
}

/// `PATCH /api/settings/profile` — write the five fields a reader owns.
///
/// The whole profile every time; see `users::update_profile` for why there is
/// no partial update. The answer is the row as it now stands, so the page
/// renders what was stored rather than what it hoped was stored.
pub(crate) async fn edit_profile(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(body): Json<EditProfile>,
) -> Response {
    let fields = match body.checked() {
        Ok(fields) => fields,
        Err(refusal) => return refuse(refusal),
    };

    match crate::users::update_profile(&state.db, session.user_id, &fields)
        .await
    {
        Ok(None) => not_found(),
        Ok(Some(row)) => {
            tracing::info!(user_id = session.user_id, "profile updated");

            response::json(
                StatusCode::OK,
                ProfileView::from(row),
                CachePolicy::NoStore,
            )
        }
        Err(cause) => {
            tracing::error!(%cause, user_id = session.user_id, "failed to update a profile");
            response::server_error()
        }
    }
}
