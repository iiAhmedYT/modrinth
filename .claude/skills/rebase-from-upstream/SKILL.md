---
name: rebase-from-upstream
description: Pull the latest changes from the upstream Modrinth monorepo (modrinth/code), rebase this fork's main onto it, and rebase the privacy-focused / offline-accounts feature branch on top, verifying the local modifications still hold. Use when syncing this fork with upstream without losing the telemetry/ad/Sentry/Intercom removals or the offline-account support.
argument-hint: [feature-branch?]
---

Refer to the modifications catalog: @.claude/skills/rebase-from-upstream/MODIFICATIONS.md

## Steps

1. **Pick the feature branch.** If `$ARGUMENTS` is set, use it. Otherwise use the current branch (run `git branch --show-current`). If the current branch is `main`, ask the user which feature branch to rebase before continuing.

2. **Confirm or add the `upstream` remote.** Run `git remote -v`. If `upstream` isn't listed, ask the user to confirm `https://github.com/modrinth/code.git` is the right URL, then `git remote add upstream <url>`. Identify upstream's default branch from `git remote show upstream` (usually `main`).

3. **Guard the working tree.** If `git status --porcelain` is non-empty, stop and ask the user whether to commit or stash. Never silently stash.

4. **Fetch and rebase `main`.**
   - `git fetch upstream`
   - `git checkout main`
   - `git rebase upstream/main`
   - If `main` conflicts, the fork has diverging commits on `main` itself — surface this and stop. Don't auto-resolve `main` conflicts; the user needs to decide.

5. **Rebase the feature branch onto the new `main`.**
   - `git checkout <feature-branch>`
   - `git rebase main`
   - Resolve each conflict using **Conflict philosophy** below. Use `git rebase --continue` after each resolution. Never `git rebase --skip` unless the user tells you to.

6. **Conflict philosophy.** Read `MODIFICATIONS.md` for the full file-by-file catalog and intent. In summary, when a conflict touches one of those files:
   - **Trackers, ads, surveys, support chat, error reporting** — refuse any upstream re-introduction. Posthog, `@sentry/vue`, `@intercom/messenger-js-sdk`, `tally.so` script tag, Aditude ad webview, `analytics/playtime`, `analytics/minecraft-server-play`, the `modrinth-download-meta` header, Modrinth+ upsell links, CSP entries for any of the above — all stay removed.
   - **`trackEvent` call sites in `.vue` files** — leave the calls in; they hit the local no-op helper. Only resolve the conflict to keep the helper *signature* intact.
   - **`settings.rs`** — `telemetry: false,` and `personalized_ads: false,` must stay forced regardless of the stored row value.
   - **`ads.rs`** — stays no-op. If upstream adds new ad commands, register them with empty `Ok(())` bodies and update `apps/app/build.rs` accordingly; never re-create the ad webview.
   - **`AccountsCard.vue` / `AddAccountModal.vue` / `auth` plugin** — preserve the offline-accounts surface. The "Add account" button must still route through `AddAccountModal`. `Credentials::new_offline` and the `is_offline()` short-circuits in `refresh()` / `online_profile()` must survive any Credentials struct refactor.
   - **`PromotionWrapper.vue`** — stays deleted. If upstream resurrects it, delete it again and strip any new imports.
   - **General rule** — when in doubt about a conflict that doesn't fit the above, prefer porting upstream's *non-tracker* refactor on top of our slim version rather than reverting to upstream's verbose version.

7. **Verify privacy posture.** From the repo root:
   ```
   grep -rn "posthog\|@sentry\|@intercom\|messenger-js-sdk\|tally\.so\|aditude" \
     apps/app-frontend/src apps/app/src packages/app-lib/src \
     apps/app-frontend/index.html apps/app/tauri.conf.json
   ```
   The only acceptable hits are `intercom-positioning.ts` and `App.vue`'s intercom-bubble CSS selectors (the layout helper, not the SDK). Anything else is a regression — fix it.

   Spot-check these specific invariants:
   - `apps/app-frontend/src/main.js` doesn't import `@sentry/vue` or call `Sentry.init`.
   - `apps/app-frontend/src/helpers/analytics.ts` doesn't import `posthog-js`; all exports are no-op bodies.
   - `apps/app-frontend/src/helpers/ads.js` exports remain `async function …() {}` no-ops.
   - `apps/app/src/api/ads.rs` doesn't contain `AD_LINK`, doesn't build a webview, and doesn't spawn a refresh loop.
   - `packages/app-lib/src/state/settings.rs` returns `telemetry: false,` and `personalized_ads: false,`.
   - `packages/app-lib/src/util/fetch.rs` keeps `let download_meta_header: Option<(String, String)> = None;`.
   - `packages/app-lib/src/api/profile/mod.rs` contains neither `analytics/playtime` nor `analytics/minecraft-server-play`.

8. **Verify offline-accounts surface.**
   - `apps/app/build.rs` lists `"login_offline"` in the `auth` plugin's `.commands(&[…])`.
   - `apps/app/src/api/auth.rs` registers `login_offline` in `tauri::generate_handler![…]`.
   - `packages/app-lib/src/state/minecraft_auth.rs` still defines `Credentials::new_offline`, `is_offline`, `offline_uuid_for_username`, `validate_offline_username`, and the `OFFLINE_REFRESH_TOKEN_MARKER` constant.
   - `apps/app-frontend/src/components/ui/AccountsCard.vue` opens `AddAccountModal` from `openAddAccount` instead of calling `login_flow()` directly.
   - Root `Cargo.toml` and `packages/app-lib/Cargo.toml` both list `md-5`.

9. **Smoke-build before pushing.** Ask first — these are slow:
   - Frontend: `pnpm prepr` (typecheck + lint for app + web).
   - Backend: `cargo check -p theseus -p theseus_gui` from the repo root.
   Report failures with file:line locations; fix them by re-applying the appropriate conflict philosophy rule.

10. **Push only after the user confirms.** Rebase rewrote history, so the push must be:
    ```
    git push --force-with-lease origin <feature-branch>
    ```
    Use `--force-with-lease`, never plain `--force`. If the lease check fails, surface it to the user — don't override.
