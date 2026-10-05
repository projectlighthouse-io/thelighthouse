//! The five handlers, and nothing else.
//!
//! They hand the pool to `store` and nothing else — no statement, no column
//! name, no constraint name. A handler that knew one would have to change when
//! the schema does.

use axum::{
    Extension,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};

use super::{
    article::Article,
    payload::{
        AuthorFilter, EditArticleRequest, NewArticleRequest, TopicFilter,
        slugify, validate_article,
    },
    refusal::{Refusal, refuse, refuse_all},
    store::{self, Filters, StoreError},
};
use crate::{
    api::AppState,
    cache::CachePolicy,
    middleware,
    request::{JsonBody, ListQuery, PageSize},
    response::{self, PaginatedResponse, json},
    session::Session,
};

/// An article carries its whole body, which is a page of prose rather than a
/// row — so a page of them is smaller than a page of notes.
///
/// The listing returning bodies at all is what lets /blog render an excerpt
/// without a second request per article. Twenty of those is already most of a
/// megabyte at the limit `payload::MAX_BODY` allows.
const PAGE: PageSize = PageSize {
    default: 10,
    max: 25,
};

/// Which listing a request is asking for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shelf {
    /// Live articles only, the same bytes for everybody.
    Public,
    /// One author's own, taken down ones included — and only ever the reader's
    /// own, by construction: the id is the one they signed in as.
    Own(i64),
}

/// **The whole check.** A taken down article is visible to exactly one person,
/// and this is where that is decided — not in the query, not in the frontend.
///
/// Both halves have to be present and equal: `?author=` naming somebody else
/// is a public listing however the reader signed in, and a reader with no
/// session gets the public listing whoever they asked about.
pub(crate) fn shelf(author: Option<i64>, reader: Option<i64>) -> Shelf {
    match (author, reader) {
        (Some(author), Some(reader)) if author == reader => Shelf::Own(author),
        _ => Shelf::Public,
    }
}

/// Every live article, newest first: all of them, one topic's, or one
/// author's — and an author reading their own gets their taken down ones too.
///
/// **The session is looked at, and that is what `?author=` costs.** The
/// listing was the same bytes for everybody and the edge could hold it; it
/// still can for every request that is not an author reading their own shelf,
/// which is why the policy is chosen per answer rather than per route. The
/// author's own view is `no-store` — it carries takedown reasons nobody else
/// may read. See this module's `mod.rs`.
///
/// Three `Query` extractors, not one struct with five fields: paging and
/// search are `request::ListQuery`'s and every listing has them, while the
/// topic and the author are this endpoint's alone.
pub(crate) async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    Query(filter): Query<TopicFilter>,
    Query(by): Query<AuthorFilter>,
) -> Response {
    let paging = query.paging(PAGE);
    let pattern = query.pattern();
    let pattern = pattern.as_deref();

    let topic = match filter.topic() {
        Ok(topic) => topic,
        Err(cause_of) => return refuse(cause_of),
    };

    let author = match by.author() {
        None => None,
        Some(username) => match store::user_id_of(&state.db, username).await {
            Ok(Some(id)) => Some(id),
            // A username nobody has matches nothing, which is an empty page
            // rather than a refusal — see `payload::AuthorFilter`.
            Ok(None) => {
                return json(
                    StatusCode::OK,
                    PaginatedResponse::new(Vec::<Article>::new(), paging, 0),
                    CachePolicy::reader_content(),
                );
            }
            Err(error) => {
                tracing::error!(?error, "failed to resolve the author");
                return response::server_error();
            }
        },
    };

    // Optional, because this route serves anonymous readers too — the session
    // only ever changes the answer when they asked about themselves.
    let reader = middleware::reader::optional(&state, &headers)
        .await
        .map(|session| session.user_id);

    let filters = Filters {
        pattern,
        topic,
        author,
    };

    match shelf(author, reader) {
        Shelf::Own(user_id) => own_page(&state, user_id, filters, paging).await,
        Shelf::Public => public_page(&state, filters, paging).await,
    }
}

/// The listing anybody may read.
async fn public_page(
    state: &AppState,
    filters: Filters<'_>,
    paging: crate::request::Paging,
) -> Response {
    let Ok(articles) = store::page(&state.db, filters, paging).await else {
        tracing::error!("failed to read the articles");
        return response::server_error();
    };

    let Ok(total) = store::count(&state.db, filters).await else {
        tracing::error!("failed to count the articles");
        return response::server_error();
    };

    json(
        StatusCode::OK,
        PaginatedResponse::new(articles, paging, total),
        CachePolicy::reader_content(),
    )
}

/// The same listing as its author reads it: taken down articles included, and
/// stored nowhere.
async fn own_page(
    state: &AppState,
    user_id: i64,
    filters: Filters<'_>,
    paging: crate::request::Paging,
) -> Response {
    let Ok(articles) = store::own(&state.db, user_id, filters, paging).await
    else {
        tracing::error!(user_id, "failed to read the author's articles");
        return response::server_error();
    };

    let Ok(total) = store::own_count(&state.db, user_id, filters).await else {
        tracing::error!(user_id, "failed to count the author's articles");
        return response::server_error();
    };

    json(
        StatusCode::OK,
        PaginatedResponse::new(articles, paging, total),
        // One reader's shelf, carrying the takedown reasons only they may
        // read. The same policy `mine` gets, for the same reason.
        CachePolicy::NoStore,
    )
}

/// One article, if it is live.
///
/// A taken down article is a 404 here, exactly as a slug that never existed
/// is — so nothing outside can tell the two apart, and a 404 the edge holds
/// for a minute is a 404 for an article that was just taken down.
pub(crate) async fn show(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Response {
    match store::find(&state.db, &slug).await {
        Ok(Some(article)) => json(
            StatusCode::OK,
            article,
            // The same policy the listing gets. A cached article page that
            // outlived its takedown is the failure this window bounds.
            CachePolicy::reader_content(),
        ),
        Ok(None) => response::not_found(),
        Err(error) => {
            tracing::error!(?error, slug, "failed to read the article");
            response::server_error()
        }
    }
}

/// This author's own articles, taken down ones included.
///
/// Its own url rather than a branch inside [`list`], because the answer
/// depends on who is asking and the listing's must not — see this module's
/// `mod.rs`. `no-store` for the same reason: it is one reader's shelf, and it
/// carries the takedown reasons nobody else may read.
pub(crate) async fn mine(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(query): Query<ListQuery>,
) -> Response {
    let paging = query.paging(PAGE);
    let pattern = query.pattern();
    let filters = Filters {
        pattern: pattern.as_deref(),
        topic: None,
        author: None,
    };

    let Ok(articles) =
        store::own(&state.db, session.user_id, filters, paging).await
    else {
        tracing::error!(
            user_id = session.user_id,
            "failed to read the author's articles"
        );
        return response::server_error();
    };

    let Ok(total) = store::own_count(&state.db, session.user_id, filters).await
    else {
        tracing::error!(
            user_id = session.user_id,
            "failed to count the author's articles"
        );
        return response::server_error();
    };

    json(
        StatusCode::OK,
        PaginatedResponse::new(articles, paging, total),
        CachePolicy::NoStore,
    )
}

/// Publishes an article. Live immediately — see this module's `mod.rs`.
pub(crate) async fn create(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    JsonBody(payload): JsonBody<NewArticleRequest>,
) -> Response {
    let checked = match validate_article(
        &payload.title,
        &payload.subtitle,
        &payload.topics,
        &payload.body,
    ) {
        Ok(checked) => checked,
        Err(refused) => return refuse_all(&refused),
    };

    // The suffix that makes the slug unique. `random_state` is the same
    // generator the session id and the oauth state use; failing it is a
    // failure to publish rather than a slug with predictable randomness in it,
    // because that randomness is the only thing keeping two identical titles
    // apart.
    let Ok(entropy) = loginwith::random_state() else {
        tracing::error!("no entropy for an article slug");
        return response::server_error();
    };
    let slug = slugify(checked.title, &entropy);

    match store::insert(&state.db, session.user_id, &slug, &checked).await {
        Ok(article) => json(StatusCode::CREATED, article, CachePolicy::NoStore),
        Err(StoreError::SlugTaken) => {
            // Two writers, the same title, and the same six hex characters.
            // Logged because it should effectively never happen, and a burst
            // of these means the randomness is not random.
            tracing::error!(slug, "an article slug collided");
            refuse(Refusal::SlugTaken)
        }
        Err(StoreError::Database(error)) => {
            tracing::error!(
                %error,
                user_id = session.user_id,
                "failed to publish the article"
            );
            response::server_error()
        }
    }
}

/// Rewrites one of the author's own articles.
///
/// The slug does not move, whatever the title becomes: a url that changed
/// under a typo fix would break every link to it.
pub(crate) async fn update(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(slug): Path<String>,
    JsonBody(payload): JsonBody<EditArticleRequest>,
) -> Response {
    let checked = match validate_article(
        &payload.title,
        &payload.subtitle,
        &payload.topics,
        &payload.body,
    ) {
        Ok(checked) => checked,
        Err(refused) => return refuse_all(&refused),
    };

    match store::rewrite(&state.db, &slug, session.user_id, &checked).await {
        // The row as stored, not what was sent — so the author sees whether
        // it is still taken down.
        Ok(Some(article)) => {
            json(StatusCode::OK, article, CachePolicy::NoStore)
        }
        Ok(None) => response::not_found(),
        Err(error) => {
            tracing::error!(
                ?error,
                slug,
                user_id = session.user_id,
                "failed to update the article"
            );
            response::server_error()
        }
    }
}

pub(crate) async fn remove(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(slug): Path<String>,
) -> Response {
    match store::delete(&state.db, &slug, session.user_id).await {
        Ok(true) => {
            response::empty(StatusCode::NO_CONTENT, CachePolicy::NoStore)
        }
        Ok(false) => response::not_found(),
        Err(error) => {
            tracing::error!(
                ?error,
                slug,
                user_id = session.user_id,
                "failed to delete the article"
            );
            response::server_error()
        }
    }
}

/// `POST /api/articles/{slug}/archive` — off the public listing and its page,
/// still on the author's shelf. 404 for an article that is not theirs.
pub(crate) async fn archive(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(slug): Path<String>,
) -> Response {
    set_archived(&state, &session, &slug, true).await
}

/// `DELETE /api/articles/{slug}/archive` — back where readers can find it.
pub(crate) async fn unarchive(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(slug): Path<String>,
) -> Response {
    set_archived(&state, &session, &slug, false).await
}

/// Both directions answer with the article as stored, so the page shows what
/// the database now holds.
async fn set_archived(
    state: &AppState,
    session: &Session,
    slug: &str,
    archived: bool,
) -> Response {
    match store::archive(&state.db, slug, session.user_id, archived).await {
        Ok(Some(article)) => {
            json(StatusCode::OK, article, CachePolicy::NoStore)
        }
        Ok(None) => response::not_found(),
        Err(error) => {
            tracing::error!(
                ?error,
                slug,
                user_id = session.user_id,
                archived,
                "failed to archive the article"
            );
            response::server_error()
        }
    }
}
