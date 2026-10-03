//! A reader's public page.
//!
//! ```text
//!   GET /api/users/{username}   what a reader chose to show anybody
//! ```
//!
//! Open to everyone and the same bytes for everyone, so it is cached at the
//! edge — on the blog's short leash, because a reader who edits their bio
//! expects to see it soon after. No email: it is neither what a reader wrote
//! about themselves nor anyone else's business.

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
    routing::get,
};
use serde::Serialize;

use crate::{api::AppState, cache::CachePolicy, response, users};

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/api/users/{username}", get(show))
}

#[derive(Debug, Serialize)]
struct PublicProfileView {
    name: String,
    avatar: Option<String>,
    username: Option<String>,
    github_username: Option<String>,
    tagline: Option<String>,
    bio: Option<String>,
    company: Option<String>,
    education: Option<String>,
    location: Option<String>,
    linkedin_url: Option<String>,
    x_url: Option<String>,
    website_url: Option<String>,
}

impl From<users::PublicProfile> for PublicProfileView {
    fn from(row: users::PublicProfile) -> Self {
        let p = row.profile;

        Self {
            name: row.name,
            avatar: row.avatar_url,
            username: p.username,
            github_username: p.github_username,
            tagline: p.tagline,
            bio: p.bio,
            company: p.company,
            education: p.education,
            location: p.location,
            linkedin_url: p.linkedin_url,
            x_url: p.x_url,
            website_url: p.website_url,
        }
    }
}

/// `GET /api/users/{username}` — one reader's public profile.
async fn show(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Response {
    match users::public_profile(&state.db, &username).await {
        Ok(None) => response::not_found(),
        Ok(Some(row)) => response::json(
            StatusCode::OK,
            PublicProfileView::from(row),
            CachePolicy::reader_content(),
        ),
        Err(cause) => {
            tracing::error!(%cause, %username, "failed to read a public profile");
            response::server_error()
        }
    }
}
