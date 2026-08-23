//! Google's wire format, and the mapping onto [`SocialUser`].

use serde::Deserialize;

use crate::{client::Client, error::Error, user::SocialUser};

/// Path under the provider's API host. Google is an `OpenID` Connect provider
/// and this is the standard userinfo endpoint.
const USERINFO: &str = "/oauth2/v3/userinfo";

pub(crate) async fn user(
    client: &Client,
    token: &str,
) -> Result<SocialUser, Error> {
    let url = format!("{}{USERINFO}", client.api());
    let profile: Profile = client.get(&url, token, "Bearer").await?;

    Ok(SocialUser {
        id: profile.sub,
        nickname: profile.nickname,
        name: profile.name,
        email: profile.email,
        avatar: profile.picture,
    })
}

/// The `OpenID` Connect claim names, which is what the userinfo endpoint
/// returns.
///
/// `email_verified` is read from the response but not carried onto
/// [`SocialUser`]: it is Google's own account, and Google does not hand out an
/// unverified address on a `email` scope grant.
#[derive(Deserialize)]
struct Profile {
    /// The subject claim. Stable per Google account per client, and the only
    /// field safe to key a user record on — an address can change hands.
    sub: String,
    nickname: Option<String>,
    name: Option<String>,
    email: Option<String>,
    picture: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, method, path},
    };

    use super::*;
    use crate::{
        client::Callback,
        provider::Provider,
        testing::{CODE, STATE, TOKEN, client_for},
    };

    #[test]
    fn sub_becomes_the_id_and_picture_becomes_the_avatar() {
        let profile: Profile = serde_json::from_str(
            r#"{"sub":"10769150350006150715113082367","name":"Ada","email":"ada@g.test",
                "email_verified":true,"picture":"https://lh3.test/a.png"}"#,
        )
        .unwrap();

        assert_eq!(profile.sub, "10769150350006150715113082367");
        assert_eq!(profile.picture.as_deref(), Some("https://lh3.test/a.png"));
    }

    #[test]
    fn a_missing_nickname_is_not_an_error() {
        // Google's userinfo usually omits it entirely, unlike GitHub's `login`.
        let profile: Profile =
            serde_json::from_str(r#"{"sub":"1","name":"Ada"}"#).unwrap();

        assert_eq!(profile.nickname, None);
        assert_eq!(profile.email, None);
        assert_eq!(profile.picture, None);
    }

    /// Socialite: `GoogleProviderTest::test_it_can_map_a_user_from_an_access_token`,
    /// which asserts the same URL and `Bearer` header.
    #[tokio::test]
    async fn a_userinfo_response_becomes_a_social_user() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": TOKEN,
                "expires_in": 3599,
                "scope": "openid profile email",
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(USERINFO))
            // `Bearer …`, where GitHub wants `token …`.
            .and(header("authorization", format!("Bearer {TOKEN}").as_str()))
            .and(header("accept", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "sub": "10769150350006150715113082367",
                "name": "Ada",
                "email": "ada@g.test",
                "email_verified": true,
                "picture": "https://lh3.test/a.png",
            })))
            .expect(1)
            .mount(&server)
            .await;

        let user = client_for(Provider::Google, &server)
            .user(Callback {
                code: CODE,
                state: STATE,
                expected_state: STATE,
            })
            .await
            .unwrap();

        assert_eq!(
            user,
            SocialUser {
                id: "10769150350006150715113082367".to_owned(),
                // Google sends no nickname; only GitHub has a `login`.
                nickname: None,
                name: Some("Ada".to_owned()),
                email: Some("ada@g.test".to_owned()),
                avatar: Some("https://lh3.test/a.png".to_owned()),
            }
        );
    }
}
