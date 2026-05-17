# Production runtime environment contract

OpenGitHub production is split into a Rust API task, a Next.js web task, and optional background worker tasks. Staging and production tasks must set `APP_ENV=staging` or `APP_ENV=production`; `ENVIRONMENT` may be used equivalently. The API also treats `NODE_ENV=production` as deployed for compatibility with mixed runtimes.

## API ECS task

Required in staging/production:

| Variable | Source | Notes |
| --- | --- | --- |
| `APP_ENV` | task env | `staging` or `production`; enables fail-fast validation. |
| `PORT` | task env | API listen port. Defaults to `3016` when unset. |
| `APP_URL` | task env | Browser-facing web origin, e.g. `https://opengithub.namuh.co`. Must be HTTPS. |
| `PUBLIC_APP_URL` | task env | Same browser-facing origin for compatibility with web/client config. Must be HTTPS. |
| `API_URL` | task env | Browser/API origin used for OAuth callbacks, e.g. `https://api.opengithub.namuh.co` or the API path origin. Must be HTTPS. |
| `DATABASE_URL` | Secrets Manager/SSM | RDS Postgres connection string. |
| `DB_SSL` | task env | Set `true` for RDS. |
| `SESSION_SECRET` | Secrets Manager/SSM | Random 32+ byte signing secret. |
| `SESSION_COOKIE_NAME` | task env | Defaults to `__Host-session`; keep that for HTTPS root-path cookies. |
| `SESSION_COOKIE_SECURE` | task env | Must be `true` in staging/production. The API fails fast if explicitly false. |
| `AUTH_GOOGLE_ID` | Secrets Manager/SSM | Google OAuth client ID. |
| `AUTH_GOOGLE_SECRET` | Secrets Manager/SSM | Google OAuth client secret. |
| `OPENGITHUB_GIT_STORAGE_DIR` | task env / volume | Durable git object storage path for the current local-bare phase. |
| `OPENAI_API_KEY` | Secrets Manager/SSM | Required when AI features are enabled. |
| `ACTIONS_SECRETS_KEY` | Secrets Manager/SSM | Required when Actions secret storage is enabled. |

Boot behavior: `crates/api/src/main.rs` validates production configuration before binding the listener. Missing required values, non-HTTPS deployed URLs, invalid `PORT`, or `SESSION_COOKIE_SECURE=false` terminate startup with a clear error.

## Web ECS task

Required in staging/production:

| Variable | Source | Notes |
| --- | --- | --- |
| `APP_ENV` | task env | `staging` or `production`; enables fail-fast validation. |
| `PORT` | task env | Web listen port. `npm start` passes this to `next start`, defaulting to `3015`. |
| `APP_URL` | task env | Browser-facing web origin. Must be HTTPS. |
| `PUBLIC_APP_URL` | task env | Browser-facing web origin. Must be HTTPS. |
| `NEXT_PUBLIC_APP_URL` | task env | Client-exposed app origin for browser bundles. |
| `API_URL` | task env | Server-side API origin. Must be HTTPS. |
| `NEXT_PUBLIC_API_URL` | task env | Client-exposed API origin when client components need direct API links. |
| `SESSION_SECRET` | Secrets Manager/SSM | Kept aligned with API task. |
| `SESSION_COOKIE_SECURE` | task env | Must be `true` in staging/production. |
| `AUTH_GOOGLE_ID` | Secrets Manager/SSM | Kept aligned with API task. |
| `AUTH_GOOGLE_SECRET` | Secrets Manager/SSM | Kept aligned with API task. |

The web build is configured with `output: "standalone"` for production images. `npm start` runs `web/scripts/validate-production-env.mjs` before `next start` so staging/production tasks fail fast instead of serving with local defaults.

## Worker ECS task

Workers should receive the same server-side runtime contract as the API task, excluding listener-only values if a worker binary does not bind HTTP:

- `APP_ENV`
- `DATABASE_URL`
- `DB_SSL=true`
- `APP_URL`
- `PUBLIC_APP_URL`
- `API_URL`
- `SESSION_SECRET`
- `SESSION_COOKIE_SECURE=true`
- feature secrets used by jobs (`OPENAI_API_KEY`, `ACTIONS_SECRETS_KEY`, storage credentials)

## Deployment pipeline wiring checklist

1. Inject secrets from AWS Secrets Manager or SSM Parameter Store, not plaintext task definitions.
2. Set `SESSION_COOKIE_SECURE=true` for every staging/production API, web, and worker task.
3. Set `PORT` explicitly in ECS task definitions (`3016` API, `3015` web) or rely on the documented defaults.
4. Register the Google OAuth callback matching `API_URL`: `${API_URL}/api/auth/google/callback`.
5. Run `make build` in CI to verify the Rust release binary and Next.js standalone output.
