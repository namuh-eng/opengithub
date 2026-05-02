# theme-001 — Site-wide Theme, Accessibility, and Responsive Layout

## Actual structure

- `crates/api/migrations/202605030042_user_appearance_settings.*.sql`
  - Adds `user_settings(user_id, theme, font_size, created_at, updated_at)` with constrained theme/font values and `set_updated_at` trigger.
- `crates/api/src/domain/personal_settings.rs`
  - Adds read/update domain behavior for authenticated appearance settings, default seeding, validation, and `appearance.settings.update` audit events.
- `crates/api/src/routes/users.rs`
  - Adds `GET/PATCH /api/user/settings/appearance` beside existing profile settings routes.
- `web/src/lib/api.ts`, `web/src/lib/server-session.ts`, `web/src/lib/theme.ts`
  - Adds typed appearance DTOs, cookie-backed API helpers, and theme/font normalization plus html data-attribute mapping.
- `web/src/app/layout.tsx`, `web/src/components/ThemeClientScript.tsx`
  - Applies `data-color-mode`, `data-light-theme`, `data-dark-theme`, and `data-font-size` on `<html>` from authenticated `user_settings` first, then `color_mode`/`font_size` cookies for unauthenticated visitors. Client listener resolves `prefers-color-scheme` and updates live after saves.
- `web/src/app/og.css`, `web/src/app/globals.css`
  - Extends Editorial tokens for light, dark, dark dimmed, and dark high contrast via `<html>` data attributes; maps Tailwind v4 token colors and sm/md/lg/xl/2xl breakpoints; adds reduced-motion handling.
- `web/src/app/settings/appearance/page.tsx`, `web/src/app/settings/appearance/actions/route.ts`, `web/src/components/AppearanceSettingsForm.tsx`
  - Replaces the placeholder with real theme/font controls, a Show preview panel, same-origin persistence, cookies, dirty-state save/reset, inline error/status, and no dead controls.
- `crates/api/tests/personal_profile_settings_contract.rs`, `web/tests/appearance-settings.test.tsx`
  - Covers API default/read/update/validation/persistence and UI rendering, save payloads, live theme application, cookie fallback normalization, and data-attribute mapping.

## Phase evidence

1. Backend/data behavior complete: additive migration plus authenticated Rust API persists `theme` and `font_size` in `user_settings`, validates supported values, and audits updates.
2. Cross-cutting theme behavior complete: root layout sets the requested html data attributes; unauthenticated visitors fall back to cookies and system mode respects `prefers-color-scheme`.
3. `/settings/appearance` UI complete: Editorial settings form supports light/dark/system/dark dimmed/dark high contrast, small/medium/large font sizes, preview show/hide, save/reset, cookies, and persisted user settings.
4. Responsive/accessibility support complete: Tailwind token/breakpoint mapping remains token-based, reduced motion is respected, controls are labeled, and tests assert no inert links/buttons.
5. Verification collected: focused Rust/API and web tests, `cd web && npx tsc --noEmit --pretty false`, Editorial banned-value scan, `CARGO_INCREMENTAL=0 make check`; `CARGO_INCREMENTAL=0 make test` is the final gate for commit.
