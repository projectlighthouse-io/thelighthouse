//! GitHub's wire format, and the mapping onto [`SocialUser`].

use serde::Deserialize;

use crate::{client::Client, error::Error, user::SocialUser};

/// GitHub wants `Authorization: token …` rather than `Bearer …`.
const SCHEME: &str = "token";

pub(crate) async fn user(client: &Client, token: &str) -> Result<SocialUser, Error> {
    let profile: Profile = client
        .get(&format!("{}/user", client.api()), token, SCHEME)
        .await?;

    let email = match profile.email {
        Some(email) => Some(email),
        // Only reachable when the reader hides their address, so a failure here
        // is not fatal: it costs the email, not the login.
        None => primary_email(client, token).await.unwrap_or_default(),
    };

    Ok(SocialUser {
        id: profile.id.to_string(),
        nickname: Some(profile.login),
        name: profile.name,
        email,
        avatar: profile.avatar_url,
    })
}

async fn primary_email(client: &Client, token: &str) -> Result<Option<String>, Error> {
    let emails: Vec<Address> = client
        .get(&format!("{}/user/emails", client.api()), token, SCHEME)
        .await?;

    Ok(primary_verified(emails))
}

/// Primary *and* verified, or nothing.
///
/// Verified is not optional here. An unverified address is one the account
/// holder typed, not one they proved they own, and downstream this address is
/// the key that links a fresh login to an existing account — so accepting an
/// unverified one lets anybody claim somebody else's account by naming their
/// address on a throwaway GitHub profile.
fn primary_verified(emails: Vec<Address>) -> Option<String> {
    emails
        .into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email)
}

#[derive(Deserialize)]
struct Profile {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

/// One entry from `/user/emails`. Named for the row rather than the field so
/// `email` can keep the wire name serde matches on.
#[derive(Deserialize)]
struct Address {
    email: String,
    primary: bool,
    verified: bool,
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

    fn emails(json: &str) -> Vec<Address> {
        serde_json::from_str(json).unwrap()
    }

    /// Mounts the token exchange so a test can get to the profile read.
    async fn mount_token(server: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": TOKEN,
            })))
            .mount(server)
            .await;
    }

    async fn sign_in(server: &MockServer) -> Result<SocialUser, Error> {
        client_for(Provider::Github, server)
            .user(Callback {
                code: CODE,
                state: STATE,
                expected_state: STATE,
            })
            .await
    }

    #[test]
    fn a_profile_keeps_only_the_fields_that_are_mapped() {
        let profile: Profile = serde_json::from_str(
            r#"{"id":1234,"login":"octocat","name":"Mona","email":"mona@github.test",
                "avatar_url":"https://avatars.test/o.png","node_id":"ignored"}"#,
        )
        .unwrap();

        assert_eq!(profile.id.to_string(), "1234");
        assert_eq!(profile.login, "octocat");
        assert_eq!(profile.email.as_deref(), Some("mona@github.test"));
        assert_eq!(
            profile.avatar_url.as_deref(),
            Some("https://avatars.test/o.png")
        );
    }

    #[test]
    fn a_profile_survives_a_hidden_email_and_no_name() {
        let profile: Profile = serde_json::from_str(
            r#"{"id":1,"login":"ghost","name":null,"email":null,"avatar_url":null}"#,
        )
        .unwrap();

        assert_eq!(profile.name, None);
        assert_eq!(profile.email, None);
    }

    #[test]
    fn the_primary_verified_address_wins_over_the_others() {
        let picked = primary_verified(emails(
            r#"[{"email":"old@x.test","primary":false,"verified":true},
                {"email":"me@x.test","primary":true,"verified":true}]"#,
        ));

        assert_eq!(picked.as_deref(), Some("me@x.test"));
    }

    #[test]
    fn an_unverified_primary_is_not_taken() {
        // Neither is the verified-but-secondary one below it: falling back to
        // that would sign the reader in as an address they did not choose.
        let picked = primary_verified(emails(
            r#"[{"email":"unverified@x.test","primary":true,"verified":false},
                {"email":"old@x.test","primary":false,"verified":true}]"#,
        ));

        assert_eq!(picked, None);
    }

    #[test]
    fn no_addresses_at_all_is_not_an_error() {
        assert_eq!(primary_verified(emails("[]")), None);
    }

    /// Socialite's equivalent asserts the URL and the `token` auth scheme
    /// against a mocked client; this asserts them against a real request.
    #[tokio::test]
    async fn a_profile_becomes_a_social_user() {
        let server = MockServer::start().await;
        mount_token(&server).await;

        Mock::given(method("GET"))
            .and(path("/user"))
            // `token …`, not `Bearer …`. Google is the one that wants Bearer.
            .and(header("authorization", format!("token {TOKEN}").as_str()))
            .and(header("accept", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 1234,
                "login": "octocat",
                "name": "Mona",
                "email": "mona@github.test",
                "avatar_url": "https://avatars.test/o.png",
            })))
            .expect(1)
            .mount(&server)
            .await;

        let user = sign_in(&server).await.unwrap();

        assert_eq!(
            user,
            SocialUser {
                id: "1234".to_owned(),
                nickname: Some("octocat".to_owned()),
                name: Some("Mona".to_owned()),
                email: Some("mona@github.test".to_owned()),
                avatar: Some("https://avatars.test/o.png".to_owned()),
            }
        );
    }

    #[tokio::test]
    async fn a_hidden_email_is_fetched_from_the_addresses_endpoint() {
        let server = MockServer::start().await;
        mount_token(&server).await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 1, "login": "ghost", "name": null, "email": null, "avatar_url": null,
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user/emails"))
            .and(header("authorization", format!("token {TOKEN}").as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"email": "old@x.test", "primary": false, "verified": true},
                {"email": "me@x.test", "primary": true, "verified": true},
            ])))
            // Only when the profile withheld one. A second call on every login
            // would be a wasted round trip against a rate-limited API.
            .expect(1)
            .mount(&server)
            .await;

        let user = sign_in(&server).await.unwrap();

        assert_eq!(user.email.as_deref(), Some("me@x.test"));
    }

    #[tokio::test]
    async fn a_visible_email_does_not_trigger_a_second_request() {
        let server = MockServer::start().await;
        mount_token(&server).await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 1, "login": "octocat", "name": null,
                "email": "mona@github.test", "avatar_url": null,
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user/emails"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(0)
            .mount(&server)
            .await;

        assert_eq!(
            sign_in(&server).await.unwrap().email.as_deref(),
            Some("mona@github.test")
        );
    }

    #[tokio::test]
    async fn a_failing_addresses_call_costs_the_email_not_the_login() {
        let server = MockServer::start().await;
        mount_token(&server).await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 7, "login": "ghost", "name": null, "email": null, "avatar_url": null,
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/user/emails"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let user = sign_in(&server).await.unwrap();

        assert_eq!(user.id, "7");
        assert_eq!(user.email, None);
    }
}
