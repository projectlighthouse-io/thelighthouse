//! One function per route. No SQL, and no rendering decisions.

use std::collections::HashMap;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
};
use ohara::{
    catalog::{ProjectEntry, Snapshot, TaskEntry},
    project::UnlockMode,
};
use uuid::Uuid;

use super::{
    entitlement::{self, is_paid},
    payload::Attempt,
    progress::{Outcome, Progress, Status},
    refusal::{Refusal, refuse},
    store,
    view::{
        AttemptView, HintDetail, HintSummary, ProgressView, ProjectDetail,
        ProjectPage, ProjectSummary, ReaderView, RestartView, StatsView,
        TaskPage, TaskProgressRow, TaskProgressView, TaskView,
    },
};
use crate::{
    api::AppState,
    books::entitlement::Access,
    cache::CachePolicy,
    request::{ListQuery, PageSize},
    response::{self, PaginatedResponse, not_found},
    tokens::Holder,
};

/// A project row is small, so the page can afford to be big — luxctl asks for
/// fifty by default and there are sixteen projects.
const PAGE: PageSize = PageSize {
    default: 50,
    max: 100,
};

/// One reader's progress across one project, keyed by task id.
type Board = HashMap<Uuid, Progress>;

/// A handler that gave up, boxed.
///
/// `Response` is large and every one of these is the unhappy path, so the
/// `Result` stays small enough that clippy does not have to be told about it.
type Failed = Box<Response>;

/// `GET /api/v1/ping`.
///
/// The one endpoint that answers nothing about anybody. It is behind the
/// signature rather than in front of it so `lux doctor` can tell "the
/// signature is wrong" from "the network is down".
pub(crate) async fn ping() -> Response {
    response::json(
        StatusCode::OK,
        serde_json::json!({ "message": "pong" }),
        CachePolicy::NoStore,
    )
}

/// `GET /api/v1/health` — what `lux doctor` asks.
///
/// Not the same endpoint as `/health`, which is the container's own probe and
/// is deliberately unrouted by caddy. This one is luxctl's, and it exists
/// because the shipped CLI calls it: `doctor` prints "healthcheck ✓" from this
/// response, and without it the first thing a reader runs when something is
/// wrong reports the api as down.
///
/// It checks the database for the same reason `/health` does — an api that
/// cannot reach postgres cannot record a single attempt, and telling somebody
/// diagnosing a problem that everything is fine is worse than telling them
/// nothing.
///
/// The field names are luxctl's `HealthCheckResponse`, which is why they are
/// what they are.
pub(crate) async fn health(State(state): State<AppState>) -> Response {
    let (status, code) = if crate::db::is_reachable(&state.db).await {
        ("ok", StatusCode::OK)
    } else {
        ("degraded", StatusCode::SERVICE_UNAVAILABLE)
    };

    response::json(
        code,
        serde_json::json!({
            "status": status,
            "app": "projectlighthouse",
            "version": env!("CARGO_PKG_VERSION"),
        }),
        CachePolicy::NoStore,
    )
}

/// `GET /api/v1/projects` — every published project.
///
/// No progress, however the caller is signed in: this is a listing, and
/// filling it in would be a query per project to answer something the CLI asks
/// properly on its next request anyway.
pub(crate) async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Response {
    let snapshot = state.catalog.current();
    let paging = query.paging(PAGE);

    let all: Vec<ProjectSummary<'_>> =
        snapshot.projects().map(ProjectSummary::of).collect();
    let total = i64::try_from(all.len()).unwrap_or(i64::MAX);

    // Paged in memory, and only here: the catalogue *is* the whole set, so
    // there is no query to push a LIMIT into and sixteen rows is not a reason
    // to invent one.
    let items = all
        .into_iter()
        .skip(usize::try_from(paging.offset).unwrap_or(usize::MAX))
        .take(usize::try_from(paging.per_page).unwrap_or(usize::MAX))
        .collect();

    response::json(
        StatusCode::OK,
        PaginatedResponse::new(items, paging, total),
        CachePolicy::NoStore,
    )
}

/// `GET /api/v1/projects/{identifier}` — one project, its blueprint, its tasks.
///
/// The response luxctl works from for the rest of a session, which is why the
/// blueprint rides along: fetching it separately would be a second signed round
/// trip for a file that is always wanted with the project.
pub(crate) async fn show(
    State(state): State<AppState>,
    holder: Option<Extension<Holder>>,
    Path(identifier): Path<String>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some(entry) = snapshot.project_by(&identifier) else {
        return not_found();
    };
    let slug = &entry.project.slug;

    let overview = match state.catalog.overview(slug) {
        Ok(overview) => overview,
        Err(cause) => {
            tracing::error!(%slug, %cause, "failed to read an overview");
            return response::server_error();
        }
    };

    let blueprint = match state.catalog.blueprint(slug) {
        Ok(blueprint) => blueprint,
        Err(cause) => {
            // Not served as an absent blueprint: luxctl would run a project
            // with no phases in it and report every task passing.
            tracing::error!(%slug, %cause, "failed to read a blueprint");
            return response::server_error();
        }
    };

    let tasks = match tasks_of(&state, entry, reader(holder.as_ref())).await {
        Ok(tasks) => tasks,
        Err(failed) => return *failed,
    };

    response::json(
        StatusCode::OK,
        ProjectDetail::of(entry, overview, blueprint, tasks),
        CachePolicy::NoStore,
    )
}

/// `GET /api/v1/projects/{identifier}/tasks` — the tasks alone.
///
/// The same list [`show`] embeds, without the blueprint: what `lux task list`
/// wants when it does not need 46 KB to print a table.
pub(crate) async fn tasks(
    State(state): State<AppState>,
    holder: Option<Extension<Holder>>,
    Path(identifier): Path<String>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some(entry) = snapshot.project_by(&identifier) else {
        return not_found();
    };

    match tasks_of(&state, entry, reader(holder.as_ref())).await {
        Ok(tasks) => {
            response::json(StatusCode::OK, tasks, CachePolicy::NoStore)
        }
        Err(failed) => *failed,
    }
}

/// `GET /api/v1/tasks/{identifier}` — one task, wherever it lives.
///
/// A uuid names exactly one; a slug names the first project that has one by
/// that name — see `Snapshot::task_by` for why that ambiguity is kept.
///
/// Carries the task's prose, which the project listing does not: one task is
/// the whole answer here, and a project's worth of descriptions is not.
pub(crate) async fn task(
    State(state): State<AppState>,
    holder: Option<Extension<Holder>>,
    Path(identifier): Path<String>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some((project, held)) = snapshot.task_by(&identifier) else {
        return not_found();
    };

    let description = match state
        .catalog
        .task_body(&project.project.slug, &held.task.slug)
    {
        Ok(description) => description,
        Err(cause) => {
            tracing::error!(%identifier, %cause, "failed to read a task");
            return response::server_error();
        }
    };

    let seen = match seen_by(&state, project, reader(holder.as_ref())).await {
        Ok(seen) => seen,
        Err(failed) => return *failed,
    };

    response::json(
        StatusCode::OK,
        view_of(project, held, description, seen.as_ref()),
        CachePolicy::NoStore,
    )
}

/// `GET /api/v1/user` — who this token belongs to, and their totals.
pub(crate) async fn me(
    State(state): State<AppState>,
    Extension(holder): Extension<Holder>,
) -> Response {
    let user_id = holder.user_id;

    let found = match crate::users::find(&state.db, user_id).await {
        Ok(found) => found,
        Err(cause) => {
            tracing::error!(%cause, user_id, "failed to read a reader");
            return response::server_error();
        }
    };

    // A live token pointing at a deleted reader. The token should have gone
    // with them; until something sweeps them, 404 is the honest answer.
    let Some(user) = found else {
        return not_found();
    };

    let (projects_attempted, tasks_completed, total_xp) = match store::totals(
        &state.db, user_id,
    )
    .await
    {
        Ok(totals) => totals,
        Err(cause) => {
            tracing::error!(%cause, user_id, "failed to total a reader's work");
            return response::server_error();
        }
    };

    response::json(
        StatusCode::OK,
        ReaderView {
            id: user.id,
            name: user.name,
            email: user.email,
            stats: StatsView {
                projects_attempted,
                tasks_completed,
                total_xp,
            },
        },
        CachePolicy::NoStore,
    )
}

/// `POST /api/v1/projects/attempts` — record one submission.
///
/// **Points are worked out here and nowhere else.** The ladder in the task's
/// yaml is graded against the attempt number and the minutes since this run
/// started, both read from the log. A first pass banks them and every later
/// pass banks nothing, which is what stops a loop of `lux submit` being an
/// income.
///
/// **The run is read, not received** — `count(project_restarts) + 1`.
pub(crate) async fn record(
    State(state): State<AppState>,
    Extension(holder): Extension<Holder>,
    Json(body): Json<Attempt>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some(project) = snapshot.project(&body.project_slug) else {
        return not_found();
    };
    let Some(project_id) = project.project.id else {
        return not_found();
    };

    let Some(held) = project
        .tasks()
        .find(|held| held.task.id == Some(body.task_id))
    else {
        // The task may well exist — in another project. Said as such rather
        // than 404, because "you sent the wrong pair" is actionable.
        return refuse(Refusal::TaskNotInProject);
    };

    let access =
        match entitlement::for_reader(&state.db, Some(holder.user_id)).await {
            Ok(access) => access,
            Err(cause) => {
                tracing::error!(%cause, "failed to check entitlement");
                return response::server_error();
            }
        };

    if is_paid(held, access) {
        return refuse(Refusal::TaskNotEntitled);
    }

    let (run, board) =
        match run_and_board(&state, holder.user_id, project_id).await {
            Ok(both) => both,
            Err(failed) => return *failed,
        };

    if locked(project, held, &board) {
        return refuse(Refusal::TaskLocked);
    }

    let so_far = progress_of(held, &board);

    // Already passed in *any* run. The attempt is still recorded — the log
    // stays honest about what happened — and it is worth nothing.
    let is_reattempt = so_far.best_points > 0;

    let points = if body.task_outcome == Outcome::Passed && !is_reattempt {
        held.task.points_for(so_far.next_attempt(), so_far.minutes)
    } else {
        0
    };

    match store::record(
        &state.db,
        holder.user_id,
        body.task_id,
        run,
        body.task_outcome,
        points,
        body.context(),
    )
    .await
    {
        Ok((id, created_at)) => {
            tracing::info!(
                user_id = holder.user_id,
                task = %held.task.slug,
                outcome = ?body.task_outcome,
                points,
                run,
                "attempt recorded"
            );

            response::json(
                StatusCode::CREATED,
                AttemptView {
                    id,
                    task_id: body.task_id,
                    run,
                    outcome: body.task_outcome,
                    points_achieved: points,
                    is_reattempt,
                    created_at: Some(created_at),
                },
                CachePolicy::NoStore,
            )
        }
        Err(cause) => {
            tracing::error!(%cause, user_id = holder.user_id, "failed to record an attempt");
            response::server_error()
        }
    }
}

/// `POST /api/v1/projects/{identifier}/restart` — start the project again.
///
/// Nothing is deleted. A restart appends a row, the run number goes up, and
/// every attempt of every earlier run stays exactly where it was — which is
/// what makes "how long did this take me the first time" answerable a year
/// later. Points already banked survive it; see `progress`.
pub(crate) async fn restart(
    State(state): State<AppState>,
    Extension(holder): Extension<Holder>,
    Path(identifier): Path<String>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some(project_id) = snapshot
        .project_by(&identifier)
        .and_then(|entry| entry.project.id)
    else {
        return not_found();
    };

    match store::restart(&state.db, holder.user_id, project_id).await {
        Ok((run, restarted_at)) => {
            tracing::info!(user_id = holder.user_id, %identifier, run, "project restarted");

            response::json(
                StatusCode::OK,
                RestartView {
                    run,
                    restarted_at: Some(restarted_at),
                },
                CachePolicy::NoStore,
            )
        }
        Err(cause) => {
            tracing::error!(%cause, %identifier, "failed to restart a project");
            response::server_error()
        }
    }
}

/// `GET /api/v1/tasks/{identifier}/hints` — the hints, with the words of the
/// ones this reader has paid for.
///
/// **The text of a locked hint never leaves this process.** Which hints exist,
/// what they cost and whether they are open yet are all safe to say; the words
/// are the thing being sold.
pub(crate) async fn hints(
    State(state): State<AppState>,
    Extension(holder): Extension<Holder>,
    Path(identifier): Path<String>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some((project, held, project_id, task_id)) =
        located(&snapshot, &identifier)
    else {
        return not_found();
    };

    let (_, board) =
        match run_and_board(&state, holder.user_id, project_id).await {
            Ok(both) => both,
            Err(failed) => return *failed,
        };

    let unlocked =
        match store::unlocked_hints(&state.db, holder.user_id, task_id).await {
            Ok(unlocked) => unlocked,
            Err(cause) => {
                tracing::error!(%cause, "failed to read unlocked hints");
                return response::server_error();
            }
        };

    let so_far = progress_of(held, &board);
    let _ = project;

    let hints: Vec<HintDetail> = held
        .task
        .hints
        .iter()
        .enumerate()
        .map(|(at, hint)| {
            let is_unlocked = hint.id.is_some_and(|id| unlocked.contains(&id));

            HintDetail {
                summary: HintSummary::of(hint, at, so_far),
                is_unlocked,
                text: is_unlocked.then(|| hint.text.clone()),
            }
        })
        .collect();

    response::json(StatusCode::OK, hints, CachePolicy::NoStore)
}

/// `POST /api/v1/tasks/{identifier}/hints/{hint}/unlock` — buy one hint.
///
/// The price is taken from ohara at the moment of the unlock and written onto
/// the row, so re-pricing a hint later cannot change what somebody was charged.
/// Unlocking twice costs once — see `store::unlock_hint`.
pub(crate) async fn unlock(
    State(state): State<AppState>,
    Extension(holder): Extension<Holder>,
    Path((identifier, hint_id)): Path<(String, Uuid)>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some((_, held, project_id, _)) = located(&snapshot, &identifier) else {
        return not_found();
    };

    let Some((at, hint)) = held
        .task
        .hints
        .iter()
        .enumerate()
        .find(|(_, hint)| hint.id == Some(hint_id))
    else {
        return not_found();
    };

    let (_, board) =
        match run_and_board(&state, holder.user_id, project_id).await {
            Ok(both) => both,
            Err(failed) => return *failed,
        };

    let so_far = progress_of(held, &board);

    if !hint.offerable(so_far.minutes, so_far.attempts_so_far()) {
        return refuse(Refusal::HintNotAvailable);
    }

    match store::unlock_hint(
        &state.db,
        holder.user_id,
        hint_id,
        hint.points_deduction,
    )
    .await
    {
        // Whether this call is what took the points or the reader already held
        // the hint, the answer is a 200 with the words in it: luxctl retries,
        // and a retry that refused would read as the hint being lost.
        Ok(_) => response::json(
            StatusCode::OK,
            HintDetail {
                summary: HintSummary::of(hint, at, so_far),
                is_unlocked: true,
                text: Some(hint.text.clone()),
            },
            CachePolicy::NoStore,
        ),
        Err(cause) => {
            tracing::error!(%cause, "failed to unlock a hint");
            response::server_error()
        }
    }
}

/// The project, task and both ids behind a task identifier.
fn located<'s>(
    snapshot: &'s Snapshot,
    identifier: &str,
) -> Option<(&'s ProjectEntry, &'s TaskEntry, Uuid, Uuid)> {
    let (project, held) = snapshot.task_by(identifier)?;

    Some((project, held, project.project.id?, held.task.id?))
}

/// The reader behind an optional bearer layer, if there was one.
fn reader(holder: Option<&Extension<Holder>>) -> Option<i64> {
    holder.map(|Extension(holder)| holder.user_id)
}

/// This reader's run, and their progress across the whole project.
///
/// One place, because every authenticated handler wants both and reading them
/// apart would let one be for run 3 while the other is for run 2.
async fn run_and_board(
    state: &AppState,
    user_id: i64,
    project_id: Uuid,
) -> Result<(i16, Board), Failed> {
    let run = store::current_run(&state.db, user_id, project_id)
        .await
        .map_err(|cause| {
            tracing::error!(%cause, "failed to read the current run");
            Box::new(response::server_error())
        })?;

    let board = store::progress(&state.db, user_id, project_id, run)
        .await
        .map_err(|cause| {
            tracing::error!(%cause, "failed to read progress");
            Box::new(response::server_error())
        })?;

    Ok((run, board))
}

/// What a caller may see of a project: their progress, and what they hold.
///
/// `None` for an anonymous caller, which is a different statement from a board
/// of zeroes — a signed-out reader has no progress, rather than no attempts.
struct Seen {
    board: Board,
    access: Access,
}

async fn seen_by(
    state: &AppState,
    project: &ProjectEntry,
    reader: Option<i64>,
) -> Result<Option<Seen>, Failed> {
    let Some((user_id, project_id)) = reader.zip(project.project.id) else {
        return Ok(None);
    };

    let access =
        entitlement::for_reader(&state.db, reader)
            .await
            .map_err(|cause| {
                tracing::error!(%cause, "failed to check entitlement");
                Box::new(response::server_error())
            })?;

    let (_, board) = run_and_board(state, user_id, project_id).await?;

    Ok(Some(Seen { board, access }))
}

/// Every task of a project, with this reader's progress folded in.
///
/// Two queries for the whole project rather than two per task, and none at all
/// for an anonymous caller.
async fn tasks_of<'p>(
    state: &AppState,
    project: &'p ProjectEntry,
    reader: Option<i64>,
) -> Result<Vec<TaskView<'p>>, Failed> {
    let seen = seen_by(state, project, reader).await?;

    Ok(project
        .tasks()
        .map(|held| view_of(project, held, None, seen.as_ref()))
        .collect())
}

/// One task as json, with progress if the caller has any.
fn view_of<'p>(
    project: &ProjectEntry,
    held: &'p TaskEntry,
    description: Option<String>,
    seen: Option<&Seen>,
) -> TaskView<'p> {
    let so_far = seen
        .map_or_else(Progress::default, |seen| progress_of(held, &seen.board));

    let hints = held
        .task
        .hints
        .iter()
        .enumerate()
        .map(|(at, hint)| HintSummary::of(hint, at, so_far))
        .collect();

    let progress = seen.map(|seen| {
        TaskProgressView::of(
            so_far,
            locked(project, held, &seen.board),
            is_paid(held, seen.access),
        )
    });

    TaskView::of(held, description, hints, progress)
}

/// This reader's progress on one task, or the empty one.
fn progress_of(held: &TaskEntry, board: &Board) -> Progress {
    held.task
        .id
        .and_then(|id| board.get(&id).copied())
        .unwrap_or_default()
}

/// Whether the sequential lock is closed on this task.
///
/// **`open` projects never lock.** That is what `unlock_mode` is for, and the
/// laravel handler applied the lock to every project regardless — so a project
/// meant to be worked in any order was not.
///
/// The first task is never locked, and the lock is decided against the task
/// immediately before it in folder order.
fn locked(project: &ProjectEntry, held: &TaskEntry, board: &Board) -> bool {
    if project.project.unlock_mode == UnlockMode::Open {
        return false;
    }

    project.task_before(&held.task.slug).is_some_and(|before| {
        progress_of(before, board).status() != Status::ChallengeCompleted
    })
}

// ------------------------------------------------- the website's own surface
//
// Unsigned, because a browser cannot carry an HMAC — the same split
// `books::routes` makes. These read from the catalogue like everything above
// and answer in the shapes `web/server/routes/_api/projects/*` maps into the
// page's own vocabulary.

/// `GET /api/projects` — the listing the projects page renders.
///
/// `Shared`: identical for a subscriber, a stranger and a crawler, which is
/// what lets the edge hold it. Progress is a separate poll, deliberately —
/// folding it in here would make this response reader-dependent and its cache
/// hit rate nothing.
pub(crate) async fn page_list(State(state): State<AppState>) -> Response {
    let snapshot = state.catalog.current();
    let projects: Vec<ProjectSummary<'_>> =
        snapshot.projects().map(ProjectSummary::of).collect();

    response::json(StatusCode::OK, projects, CachePolicy::public_content())
}

/// `GET /api/projects/{slug}` — one project's page.
///
/// No blueprint: a browser cannot run one and 46 KB of `.bp` in a page payload
/// is 46 KB nobody reads. luxctl's `/api/v1` route is where that lives.
pub(crate) async fn page_show(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some(entry) = snapshot.project(&slug) else {
        return not_found();
    };

    let overview = match state.catalog.overview(&slug) {
        Ok(overview) => overview,
        Err(cause) => {
            tracing::error!(%slug, %cause, "failed to read an overview");
            return response::server_error();
        }
    };

    response::json(
        StatusCode::OK,
        ProjectPage::of(entry, overview),
        CachePolicy::public_content(),
    )
}

/// `GET /api/projects/{slug}/progress` — what the open tab polls.
///
/// Every five seconds while the tab is visible, and not at all while it is
/// hidden — see the page. Not SSE and not a websocket: a reader working
/// through a project in their terminal generates an event every few minutes at
/// most, and a poll that costs one indexed query is cheaper to run and far
/// cheaper to reason about than a connection per tab that has to survive a
/// deploy.
///
/// Behind the session cookie rather than a token: the caller is a browser.
pub(crate) async fn progress(
    State(state): State<AppState>,
    Extension(session): Extension<crate::session::Session>,
    Path(slug): Path<String>,
) -> Response {
    let snapshot = state.catalog.current();

    let Some(entry) = snapshot.project(&slug) else {
        return not_found();
    };
    let Some(project_id) = entry.project.id else {
        return not_found();
    };

    let access =
        match entitlement::for_reader(&state.db, Some(session.user_id)).await {
            Ok(access) => access,
            Err(cause) => {
                tracing::error!(%cause, "failed to check entitlement");
                return response::server_error();
            }
        };

    let (run, board) =
        match run_and_board(&state, session.user_id, project_id).await {
            Ok(both) => both,
            Err(failed) => return *failed,
        };

    let mut completed = 0;
    let mut points_earned = 0;
    let mut tasks = Vec::new();

    for held in entry.tasks() {
        let so_far = progress_of(held, &board);

        if so_far.status() == Status::ChallengeCompleted {
            completed += 1;
        }
        points_earned += so_far.points();

        tasks.push(TaskProgressRow {
            slug: held.task.slug.clone(),
            progress: TaskProgressView::of(
                so_far,
                locked(entry, held, &board),
                is_paid(held, access),
            ),
        });
    }

    response::json(
        StatusCode::OK,
        ProgressView {
            run,
            completed,
            total: tasks.len(),
            points_earned,
            tasks,
        },
        CachePolicy::NoStore,
    )
}

/// `GET /api/projects/{slug}/tasks/{task}` — one task's brief, as a page.
///
/// Addressed within its project, unlike the luxctl route: a url a reader can
/// see should say which project they are in, and a slug that is unique inside
/// a project is not unique across the repo.
pub(crate) async fn page_task(
    State(state): State<AppState>,
    Path((slug, task)): Path<(String, String)>,
) -> Response {
    let snapshot = state.catalog.current();

    let (Some(entry), Some(held)) =
        (snapshot.project(&slug), snapshot.task(&slug, &task))
    else {
        return not_found();
    };

    let markdown = match state.catalog.task_body(&slug, &task) {
        Ok(markdown) => markdown,
        Err(cause) => {
            tracing::error!(%slug, %task, %cause, "failed to read a task");
            return response::server_error();
        }
    };

    response::json(
        StatusCode::OK,
        TaskPage::of(entry, held, markdown.as_deref()),
        CachePolicy::public_content(),
    )
}
