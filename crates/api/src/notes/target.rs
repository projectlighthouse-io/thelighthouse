//! Which lesson a note belongs to, and whether it may be a reply.
//!
//! The half of validation that needs a query. `super::payload` is the half that
//! does not.

use sqlx::postgres::PgPool;
use uuid::Uuid;

use super::{payload::NewNoteRequest, refusal::Refusal, store};

/// Where a new note is going, once its lesson and any parent have been checked.
#[derive(Debug)]
pub(crate) enum NoteTarget {
    /// `LessonId`, not `Lesson`: it carries an id, and the content endpoints
    /// bring a real `Lesson` this must not be confused with.
    LessonId(Uuid),
    /// No such book or lesson.
    Missing,
    /// The parent named cannot be replied to. Carries what to tell the reader.
    Refused(Refusal),
    /// The database could not answer. Already logged.
    Failed,
}

/// Resolves the lesson from its slugs and checks the parent in one query.
///
/// `reader` is who is replying: a parent somebody else kept private is
/// answered as missing, the same as a note id that names nothing.
pub(crate) async fn resolve(
    db: &PgPool,
    payload: &NewNoteRequest,
    reader: i64,
) -> NoteTarget {
    let found = store::lesson_and_parent(
        db,
        &payload.book,
        &payload.lesson,
        payload.parent_id,
        reader,
    )
    .await;

    let (lesson_id, parent_is_root, parent_is_here) = match found {
        Ok(Some(row)) => row,
        Ok(None) => return NoteTarget::Missing,
        Err(error) => {
            tracing::error!(
                ?error,
                book = %payload.book,
                lesson = %payload.lesson,
                "failed to resolve the lesson"
            );
            return NoteTarget::Failed;
        }
    };

    if payload.parent_id.is_none() {
        return NoteTarget::LessonId(lesson_id);
    }

    // Exists, top-level, and on this lesson. Laravel checks the first two;
    // without the third a reply lands in another lesson's thread carrying
    // offsets that mean nothing there.
    match (parent_is_root, parent_is_here) {
        (Some(true), Some(true)) => NoteTarget::LessonId(lesson_id),
        // Only reachable by someone already looking at the parent, so naming
        // the problem gives nothing away.
        (Some(false), Some(true)) => NoteTarget::Refused(Refusal::NestedReply),
        // Wrong lesson and no such note answer the same, or this becomes a
        // way to ask whether a given note id is real.
        _ => NoteTarget::Refused(Refusal::NoSuchNote),
    }
}
