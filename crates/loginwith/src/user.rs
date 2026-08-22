//! The profile, once the provider's own shape has been thrown away.

/// A profile, mapped to the same five fields Socialite exposes and the user
/// upsert reads.
///
/// No raw-response passthrough. Socialite keeps one so a caller can reach a
/// field the mapping missed; adding it here would mean every caller is free to
/// depend on a provider's wire format, which is the coupling this type exists
/// to prevent. A field worth reading is worth naming.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocialUser {
    /// The provider's own identifier. A string even for GitHub, whose ids are
    /// numeric, because it is stored and compared, never arithmetic.
    pub id: String,
    /// GitHub's `login`. Google has no equivalent and leaves this `None`.
    pub nickname: Option<String>,
    pub name: Option<String>,
    /// Optional on purpose. A GitHub account can have no verified address, and
    /// the caller has to decide what that means rather than being handed `""`.
    pub email: Option<String>,
    pub avatar: Option<String>,
}
