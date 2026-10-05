//! Which books a plan sells.
//!
//! **A plan carries its own list, by slug.** `pricing.yaml` says what each
//! price includes, `lighthouse-prices` copies it into `billing.yaml`, and this
//! is where it is read.
//!
//! It used to be the other way round: each `book.yaml` named the tracks it was
//! on, and a plan's track was parsed out of its id. That kept the pricing file
//! free of content, but it meant no single place answered "what does $99 buy" —
//! the answer was spread across every book in the catalogue, and a book joined
//! a paid bundle by a change nobody reviewing prices would see.
//!
//! The `tracks:` map has not gone anywhere. It is a reading order, and it still
//! orders and groups the books pages. It is simply no longer what decides who
//! may read what.

use billing::{PlanId, Plans, Subscription};
use ohara::catalog::Snapshot;

/// The laravel app's plan, honoured for readers still subscribed to it.
///
/// Laravel sold one product, and it opened every book. Readers who bought it
/// there keep renewing it at Stripe, so it has to keep meaning what they paid
/// for. **It is never sold here**, which is why it lives in code rather than in
/// `pricing.yaml`: a plan in that file is one the pricing page offers, and one
/// `lighthouse-prices` needs a live Stripe price for.
pub(crate) const VOYAGE: &str = "voyage";

/// The prefix of every price lookup key the laravel app sold [`VOYAGE`] on:
/// `_monthly`, `_yearly` and `_lifetime`.
const VOYAGE_PRICES: &str = "lighthouse_voyage_";

/// Whether a plan unlocks this book.
///
/// `everything` is the one thing a list cannot say, because the point of it is
/// the books that do not exist yet. Anything else is membership of the list.
///
/// [`VOYAGE`] unlocks everything without being declared, because it never is.
pub(crate) fn covers(plans: &Plans, plan: &PlanId, slug: &str) -> bool {
    if plan.as_str() == VOYAGE {
        return true;
    }

    plans.get(plan).is_some_and(|held| {
        held.everything || held.books.iter().any(|named| named == slug)
    })
}

/// The plan a subscription is recorded under.
///
/// The plan this application wrote into metadata when it has one. Failing
/// that, a laravel price is [`VOYAGE`] — the metadata is what laravel never
/// wrote, and its price is the only way left to recognise it. Anything else is
/// `unknown`, which covers nothing.
pub(crate) fn plan_of(subscription: &Subscription) -> &str {
    if let Some(plan) = subscription.plan.as_ref() {
        return plan.as_str();
    }

    match subscription.price_key.as_deref() {
        Some(key) if key.starts_with(VOYAGE_PRICES) => VOYAGE,
        _ => "unknown",
    }
}

/// Every book a plan unlocks, by the id an entitlement points at.
///
/// A book with no id is skipped rather than failing the purchase: an
/// entitlement names a book by id, so one that has never been given an id
/// cannot be granted. Silently dropping it would be worse, which is why the
/// caller logs the difference between what was bought and what was granted.
pub(crate) fn books(
    snapshot: &Snapshot,
    plans: &Plans,
    plan: &PlanId,
) -> Vec<uuid::Uuid> {
    snapshot
        .books()
        .filter(|entry| covers(plans, plan, &entry.book.slug))
        .filter_map(|entry| entry.book.id)
        .collect()
}

/// Check every plan names books that exist.
///
/// At boot, so a typo is a named startup failure. The failure it prevents is
/// quiet: a slug with a letter wrong parses, sells, charges, and grants one
/// book fewer than it promised, and the first report of it is a reader who
/// paid.
///
/// A plan that unlocks `everything` names nothing and is checked against
/// nothing. A plan that names nothing *and* is not `everything` is refused:
/// that is a price with no content behind it, and selling one is worse than
/// failing to start.
///
/// # Errors
///
/// The plan, and the slug it names that no book has.
pub(crate) fn validate(
    plans: &Plans,
    snapshot: &Snapshot,
) -> Result<(), String> {
    for plan in plans.all() {
        if plan.everything {
            continue;
        }

        if plan.books.is_empty() {
            return Err(format!(
                "the plan {} names no books and does not sell everything, so \
                 it would charge for nothing",
                plan.id
            ));
        }

        for slug in &plan.books {
            if !snapshot.books().any(|entry| &entry.book.slug == slug) {
                return Err(format!(
                    "the plan {} names the book {slug:?}, which is not in the \
                     catalogue",
                    plan.id
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> Snapshot {
        Snapshot::load(&ohara::fixture::content(), ohara::Drafts::Hidden)
            .unwrap()
    }

    /// A slug the fixture catalogue really has, so a test cannot pass by
    /// agreeing with a typo.
    fn a_real_slug(snapshot: &Snapshot) -> String {
        snapshot
            .books()
            .next()
            .expect("the fixture has a book")
            .book
            .slug
            .clone()
    }

    fn plans_from(yaml: &str) -> Plans {
        Plans::from_yaml(yaml).expect("the test yaml parses")
    }

    #[test]
    fn a_plan_covers_the_books_it_names_and_no_others() {
        let snapshot = snapshot();
        let slug = a_real_slug(&snapshot);

        let plans = plans_from(&format!(
            "plans:\n  - id: foundation_yearly\n    price: price_x\n    \
             interval: year\n    books: [{slug}]\n"
        ));

        assert!(covers(&plans, &"foundation_yearly".into(), &slug));
        assert!(!covers(&plans, &"foundation_yearly".into(), "not-a-book"));
    }

    #[test]
    fn everything_covers_a_book_the_list_could_not_have_named() {
        // The whole reason the flag exists: books that do not exist yet, and
        // the ones on no reading order, which no track ever reached.
        let plans = plans_from(
            "plans:\n  - id: all_lifetime\n    price: price_x\n    \
             interval: once\n    everything: true\n",
        );

        assert!(covers(&plans, &"all_lifetime".into(), "written-next-year"));
    }

    #[test]
    fn a_plan_nobody_declared_covers_nothing() {
        // A membership can outlive the plan it was bought on — a retired plan
        // is simply absent here, and absent must not mean unlimited.
        let plans = plans_from(
            "plans:\n  - id: foundation_yearly\n    price: price_x\n    \
             interval: year\n    everything: true\n",
        );

        assert!(!covers(&plans, &"go_yearly".into(), "anything"));
    }

    #[test]
    fn the_laravel_voyage_plan_covers_every_book() {
        // Declared nowhere, and still everything: the readers who bought it in
        // the laravel app are paying for every book.
        let plans = plans_from(
            "plans:\n  - id: foundation_yearly\n    price: price_x\n    \
             interval: year\n    books: [os-fundamentals]\n",
        );

        assert!(covers(&plans, &VOYAGE.into(), "written-next-year"));
    }

    fn subscription(
        plan: Option<&str>,
        price_key: Option<&str>,
    ) -> Subscription {
        Subscription {
            reference: "sub_1".to_owned(),
            account: None,
            customer: None,
            price_key: price_key.map(str::to_owned),
            plan: plan.map(PlanId::from),
            status: billing::Status::Active,
            period_ends_at: None,
            cancel_at: None,
        }
    }

    #[test]
    fn the_plan_in_metadata_wins_over_a_voyage_price() {
        let held = subscription(
            Some("foundation_yearly"),
            Some("lighthouse_voyage_yearly"),
        );

        assert_eq!(plan_of(&held), "foundation_yearly");
    }

    #[test]
    fn a_voyage_price_with_no_metadata_is_voyage() {
        let held = subscription(None, Some("lighthouse_voyage_monthly"));

        assert_eq!(plan_of(&held), VOYAGE);
    }

    #[test]
    fn neither_metadata_nor_a_voyage_price_is_unknown() {
        assert_eq!(plan_of(&subscription(None, None)), "unknown");
        assert_eq!(
            plan_of(&subscription(None, Some("somebody_elses_price"))),
            "unknown"
        );
    }

    #[test]
    fn books_are_granted_by_id_for_the_plan_that_names_them() {
        let snapshot = snapshot();
        let slug = a_real_slug(&snapshot);

        let plans = plans_from(&format!(
            "plans:\n  - id: one_lifetime\n    price: price_x\n    \
             interval: once\n    books: [{slug}]\n"
        ));

        let granted = books(&snapshot, &plans, &"one_lifetime".into());
        let everything = books(&snapshot, &plans, &"nope_lifetime".into());

        assert!(granted.len() <= 1, "granted more than it named");
        assert!(everything.is_empty(), "an unknown plan granted books");
    }

    #[test]
    fn a_plan_naming_a_book_that_does_not_exist_fails_the_boot() {
        // The quiet failure: a slug with a letter wrong sells, charges, and
        // grants one book fewer than it promised.
        let plans = plans_from(
            "plans:\n  - id: foundation_yearly\n    price: price_x\n    \
             interval: year\n    books: [os-fundamentalz]\n",
        );

        let refused = validate(&plans, &snapshot()).unwrap_err();

        assert!(refused.contains("os-fundamentalz"), "{refused}");
    }

    #[test]
    fn a_plan_naming_nothing_at_all_fails_the_boot() {
        // A price with no content behind it. Failing to start beats selling it.
        let plans = plans_from(
            "plans:\n  - id: empty_yearly\n    price: price_x\n    \
             interval: year\n",
        );

        assert!(validate(&plans, &snapshot()).is_err());
    }

    #[test]
    fn everything_needs_no_list_and_is_always_valid() {
        let plans = plans_from(
            "plans:\n  - id: all_lifetime\n    price: price_x\n    \
             interval: once\n    everything: true\n",
        );

        assert!(validate(&plans, &snapshot()).is_ok());
    }

    #[test]
    fn the_books_a_plan_names_are_all_in_the_catalogue() {
        let snapshot = snapshot();
        let slug = a_real_slug(&snapshot);

        let plans = plans_from(&format!(
            "plans:\n  - id: foundation_yearly\n    price: price_x\n    \
             interval: year\n    books: [{slug}]\n"
        ));

        assert!(validate(&plans, &snapshot).is_ok());
    }
}
