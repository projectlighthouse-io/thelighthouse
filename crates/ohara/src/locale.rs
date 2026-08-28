//! The languages the site is published in.

use serde::{Deserialize, Serialize};

/// A language a lesson can be read in.
///
/// An enum and not a `String`, for the reason [`super::price::Currency`] is
/// one: a typo becomes a parse error naming the file, rather than a lesson that
/// silently has no body in a language nobody ever asks for. `content_path:
/// end: lesson.md` is refused at load; as a string key it would have parsed
/// and then produced a 404 that looks like a missing file.
///
/// The set is closed and small on purpose. Adding a language is a variant here
/// and a `content_path` entry in every lesson that has one — which is the point,
/// because a language nothing is written in is not a language the site offers.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// English, and the fallback. Every lesson must have one — see
    /// [`super::lesson::Lesson`] — so resolving a body can always land
    /// somewhere rather than 404ing a reader whose language is not translated
    /// yet.
    #[default]
    En,
    /// Bengali.
    Bn,
}

impl Locale {
    /// What `*_translations.locale` stores. The column is `varchar(5)`, so
    /// every variant has to stay inside that.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Bn => "bn",
        }
    }

    /// `None` for anything that is not a language the site publishes.
    ///
    /// Takes the string form rather than a number, unlike [`super::Status`]:
    /// locale is stored as text and arrives from an HTTP header or a cookie,
    /// where the wire format is the name.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "en" => Some(Self::En),
            "bn" => Some(Self::Bn),
            _ => None,
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant, so a new one cannot be added without being listed here.
    const LOCALES: [Locale; 2] = [Locale::En, Locale::Bn];

    #[test]
    fn every_locale_survives_the_round_trip() {
        for locale in LOCALES {
            assert_eq!(Locale::parse(locale.as_str()), Some(locale));
        }
    }

    #[test]
    fn every_locale_fits_the_column() {
        // `varchar(5)`, from the laravel schema migration 0 carries.
        for locale in LOCALES {
            assert!(locale.as_str().len() <= 5, "{locale}");
        }
    }

    #[test]
    fn english_is_the_default_because_it_is_the_fallback() {
        assert_eq!(Locale::default(), Locale::En);
    }

    #[test]
    fn a_language_the_site_does_not_publish_is_not_guessed_at() {
        assert_eq!(Locale::parse("end"), None);
        assert_eq!(Locale::parse("EN"), None);
        assert_eq!(Locale::parse(""), None);
    }

    #[test]
    fn yaml_uses_the_name_not_a_number() {
        let locale: Locale = serde_norway::from_str("bn").unwrap();

        assert_eq!(locale, Locale::Bn);
        assert!(serde_norway::from_str::<Locale>("end").is_err());
    }
}
