# loginwith

OAuth 2.0 sign-in for GitHub and Google. Laravel Socialite's job, cut down to
the two providers this platform uses.

Social auth is the only way into projectlighthouse.io — no email, no password —
so this is the whole of the front door.

## What it does, and what it leaves to you

```
authorize_url ──► provider consent screen ──► your callback ──► user()
```

The crate owns no session and no database. It takes a code and gives back a
profile; deciding who that profile *is*, and keeping them signed in, is the
caller's. That split is why there is no `redirect()` here: Socialite's version
writes the state into the session for you, and this one cannot, because it has
never heard of one. You get the URL and put the state somewhere yourself.

## Using it

Register at boot, then ask for a driver by name — `Socialite::driver($provider)`.

```rust
use loginwith::{Callback, GithubProvider, GoogleProvider, random_state};

// main(), after config and before the router. Credentials are read once, here,
// and a bad one fails the boot instead of the first login attempt.
let socials = loginwith::providers([
    GithubProvider::with(c.github_id.clone(), c.github_secret.clone(), github_redirect),
    GoogleProvider::with(c.google_id.clone(), c.google_secret.clone(), google_redirect),
])?;
```

```rust
// GET /auth/login/{provider}
let driver = socials.driver(provider).ok_or_else(not_found)?;

let state = random_state()?;
session.put("oauth_state", &state);  // must survive until the callback
redirect(driver.authorize_url(&state));
```

```rust
// GET /auth/callback/{provider}
let user = socials.driver(provider).ok_or_else(not_found)?
    .user(Callback {
        code:           query.code,
        state:          query.state,
        expected_state: &session.pull("oauth_state").unwrap_or_default(),
    })
    .await?;

// user.id, user.nickname, user.name, user.email, user.avatar
```

A handler never sees a client id, and `driver` returns `None` for both an
unknown provider and one that was never registered — a handler owes the same
404 to each, and distinguishing them would confirm which.

`Providers` is cheap to clone (the drivers share one connection pool), so it
goes straight into request state.

`redirect_url` must be byte-identical to the one registered with the provider.
Both send it again at the token exchange and compare it.

### The state is not optional

`Callback` takes both states as fields so the CSRF check cannot be skipped at a
call site. Without it, anyone can point a browser at your callback URL carrying
their own `code` and sign the victim into *the attacker's* account.

The comparison is constant time, and an empty expected state never matches —
including against another empty one. A missing state means the session is gone
or none was ever issued, and neither is permission to continue.

The check runs before the code is redeemed, so a forged callback never costs a
token exchange. There is a test that asserts the server was never contacted.

## Fields

| | GitHub | Google |
|---|---|---|
| `id` | `id` | `sub` |
| `nickname` | `login` | `nickname`, which userinfo usually omits |
| `name` | `name` | `name` |
| `email` | `email`, else the primary verified address | `email` |
| `avatar` | `avatar_url` | `picture` |

All but `id` are `Option`. A GitHub account can genuinely have no name and no
reachable address; handing back `""` would only move that problem downstream.

**GitHub's email needs a second request.** A reader with a private address gets
`"email": null` from `/user`, so the crate falls back to `/user/emails` and takes
the entry that is primary **and** verified. Verified is not negotiable: that
address is what links a fresh login to an existing account, so accepting an
unverified one would let anybody claim somebody else's account by naming their
address on a throwaway profile. If the fallback request fails, it costs the
email, not the login.

## Deliberately missing

PKCE, refresh tokens, Google ID-token (JWT) verification, `stateless()`, custom
parameters, and the raw-response passthrough. Socialite carries all of it.

The raw passthrough is the one worth naming: keeping it would let every caller
depend on a provider's wire format, which is the coupling `SocialUser` exists to
prevent. A field worth reading is worth adding to the struct.

## Layout

Every module is private, with its public items re-exported from `lib.rs`, so
callers write `loginwith::Client` and moving an item between modules is not a
breaking change.

| | |
|---|---|
| `registry.rs` | registering the providers at boot, `driver` lookup |
| `provider.rs` | the two providers — endpoints, scopes, `parse` |
| `client.rs` | the flow: authorize URL, state check, token exchange |
| `github.rs`, `google.rs` | one per provider: wire format and mapping |
| `state.rs` | minting the state and comparing it |
| `user.rs`, `error.rs` | the two types that cross the crate boundary |
| `testing.rs` | scaffolding shared by the flow tests |

## Tests

```bash
cargo test -p loginwith
```

No network. The flow tests run the real code against a `wiremock` server and
assert what actually went out — the exact token-exchange body, the
`Authorization` scheme each provider wants (`token` for GitHub, `Bearer` for
Google), and that a bad state produces no request at all.

`Client::with_endpoints` is the seam that makes that possible: it is
`pub(crate)`, and it is the same one Socialite gets from test provider stubs
overriding `getTokenUrl()`.

## Notes against Socialite

- Google's token endpoint is `oauth2.googleapis.com/token`. Socialite still
  points at the older `www.googleapis.com/oauth2/v4/token` alias.
- A `User-Agent` is set on every request. reqwest sends none by default, unlike
  Guzzle, and GitHub answers 403 without one.
- The token response is checked for an `access_token`, not just a 2xx — GitHub
  reports a bad code as `200` with an error body.
- TLS is rustls with bundled webpki roots, so the alpine runtime image needs no
  `ca-certificates`.
