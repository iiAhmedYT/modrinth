# Fork modifications catalog

This catalog captures the **current** intent of every file this fork has changed relative to upstream `modrinth/code`. It is grouped by capability, not by commit, because new commits can land at any time and the rebase needs to preserve intent, not history.

**Keep this file in sync.** Whenever a new change lands that adjusts the privacy posture, the offline-account flow, or any other fork-specific behavior, update the relevant section below (or add a new one) before merging the change to a long-lived branch. If a file is touched here without a corresponding catalog entry, the next rebase will silently drop the change.

When resolving rebase conflicts, the goal is to preserve the intent listed below, not the literal diff — upstream may refactor surrounding code, and porting that refactor on top of our version is fine as long as the behavior column stays true.

## Branding rename (Modrinth App → Meverinth)

The desktop app is rebranded to **Meverinth** so users don't confuse this fork with upstream's Modrinth App. The deep-link URL schemes (`modrinth://`, `modrinthscheme://`) are deliberately kept so the fork can still handle install links from modrinth.com.

| File | Intent to preserve |
| --- | --- |
| `apps/app/tauri.conf.json` | `productName`, `mainBinaryName`, `identifier`, and the `windows[0].title` are all `Meverinth` (not `Modrinth App`/`ModrinthApp`). |
| `apps/app/tauri.macos.conf.json` | `windows[0].title` is `Meverinth`. |
| `apps/app/tauri.linux.conf.json` | `mainBinaryName` is `meverinth` (lowercase for the Linux executable). |
| `apps/app/Info.plist` | `CFBundleURLName` is `Meverinth`. `CFBundleURLSchemes` stays as `modrinth`/`modrinthscheme` — those are the deep-link schemes the app handles, not the app name. |
| `apps/app/Cargo.toml` | `description` mentions Meverinth as a privacy-focused fork of the Modrinth desktop app. |
| `apps/app/README.md` | Top-level heading and intro paragraph describe Meverinth, not Modrinth App. Linking back to upstream Modrinth App for context is fine. |
| `apps/app/COPYING.md` / `apps/app-frontend/COPYING.md` | Reference Meverinth as a fork of Modrinth App. |
| `apps/app/src/main.rs` | Tracing log message says "Loading Meverinth!" and the WebView2 error dialog says "required to run Meverinth". |
| `apps/app/src/api/oauth_utils/auth_code_reply/page.html` | `<title>` is `Sign In - Meverinth`. |
| `apps/app/nsis/hooks.nsi` | The "old installation detected" message says "Meverinth" instead of "Modrinth App". |
| `apps/app-frontend/index.html` | `<title>` is `Meverinth`. |
| `apps/app-frontend/src/App.vue` | All `Modrinth App v{version}` update-popup `defaultMessage` strings say `Meverinth v{version}`. |
| `apps/app-frontend/src/components/ui/ErrorModal.vue` | All user-facing copy says "Meverinth" (no "the Meverinth" / "The Meverinth" leftovers). |
| `apps/app-frontend/src/components/ui/modal/AppSettingsModal.vue` | About panel shows "Meverinth {{ version }}". |
| `apps/app-frontend/src/components/ui/settings/AppearanceSettings.vue` | Color-theme description references Meverinth. |
| `apps/app-frontend/src/components/ui/settings/PrivacySettings.vue` | Privacy notice opens with "Meverinth is a privacy-focused fork of the Modrinth desktop app." |
| `apps/app-frontend/src/pages/Index.vue` | First-launch greeting is "Welcome to Meverinth!". |

**Things deliberately not renamed:**
- The `ModrinthAppLogo` import name and `modrinth_app.svg` asset — we still use the upstream logo asset.
- Any `@modrinth/...` workspace package name (technical identifier, not user-facing).
- "Modrinth account", "Modrinth API", "Modrinth+", "Modrinth Hosting", "Modrinth modpack" — these refer to upstream's services, not the app.
- `modrinth://` / `modrinthscheme://` URL schemes — install links from modrinth.com use these.
- The `theseus` / `theseus_gui` crate names.

## GitHub Actions build

Ships a fork-specific workflow that builds Meverinth on free GitHub-hosted runners for Linux, macOS (arm64), and Windows. No paid runners, no S3-backed sccache, no code-signing secrets required. Runs on manual dispatch and on `v*` tag pushes; tag pushes also draft a GitHub Release with the built binaries attached.

| File | Intent to preserve |
| --- | --- |
| `.github/workflows/meverinth-build.yml` | **New file.** Matrix over `ubuntu-latest` / `macos-latest` / `windows-latest`. Copies `packages/app-lib/.env.prod` to `.env` before building. Stamps `MEVERINTH_VERSION` into `apps/app-frontend/package.json` from a dispatch input, tag name, or the existing `version` field, in that order. Uploads per-platform artifacts and, on tag push, drafts a release. |
| `.github/workflows/theseus-build.yml` | **Deleted.** Modrinth's builder; needs Blacksmith runners, sccache S3 secrets, Apple/DigiCert code-signing keys, and Tauri minisign keys. Would fire on the same triggers as our workflow and fail. |
| `.github/workflows/theseus-release.yml` | **Deleted.** Consumed the theseus-build artifacts to publish to Modrinth's own release channel. |
| `apps/app/tauri-release.conf.json` | **Deleted.** Nothing left in it worth keeping once the updater block and the DigiCert `signCommand` were removed — Tauri falls back to `tauri.conf.json`, which is what we want. |

## Auto-updates disabled

The Tauri auto-updater is turned off in release builds, and the standalone Linux update-check polling loop is removed. Without this, a release build would download and install Modrinth App releases from `launcher-files.modrinth.com` and silently replace Meverinth (Modrinth's signed artifacts validate against the pubkey that used to be baked into the binary).

| File | Intent to preserve |
| --- | --- |
| `apps/app/tauri-release.conf.json` | No `plugins.updater` block, no `updater` in `build.features`, no `updater` in `app.security.capabilities`. The Windows `signCommand` for release signing stays. |
| `apps/app/capabilities/updater.json` | **Deleted.** If upstream resurrects it, delete it again. |
| `apps/app-frontend/src/App.vue` | No `checkLinuxUpdates` function, no fetch of `https://launcher-files.modrinth.com/updates.json`, no `linuxBody` entry in `updatePopupMessages`. `checkUpdates()` returns immediately when `areUpdatesEnabled()` reports `false` (which it always does now, because the `updater` Cargo feature stays off) — do not restore the Linux polling branch. |

The rest of the update UI (progress spinner, "Download update" button, changelog link) is left in the file because it never executes at runtime — `areUpdatesEnabled()` short-circuits the whole flow — but if you want to fully strip it during a future cleanup, do it in its own commit so the diff is reviewable.

## Privacy posture

Removes telemetry, error reporting, in-app support chat, surveys, ads, and Modrinth+ upsells. Nothing leaves the machine beyond what is strictly required for account sign-in and content browsing.

| File | Intent to preserve |
| --- | --- |
| `apps/app-frontend/index.html` | No `tally.so/widgets/embed.js` script tag (or any other third-party tracker tag). |
| `apps/app-frontend/src/main.js` | No `@sentry/vue` import, no `Sentry.init` call. |
| `apps/app-frontend/src/App.vue` | No Intercom boot, no Tally survey popup, no Modrinth+ ad block, no `PromotionWrapper` import, no analytics init in `setupApp()`. The `useIntercomPositioning` composable stays — it's a layout helper, not the Intercom SDK. |
| `apps/app-frontend/src/helpers/analytics.ts` | All exports (`initAnalytics`, `debugAnalytics`, `optInAnalytics`, `optOutAnalytics`, `trackEvent`) are no-op stubs. The `AnalyticsEvent` / `AnalyticsEventMap` types stay so call sites keep type-checking. |
| `apps/app-frontend/src/helpers/ads.js` | All exports (`init_ads_window`, `show_ads_window`, `hide_ads_window`, `record_ads_click`, `open_ads_link`) are `async function …() {}` no-ops with no `invoke()` calls. |
| `apps/app-frontend/src/components/ui/settings/PrivacySettings.vue` | A privacy-build notice card plus only the Discord RPC toggle. `telemetry` and `personalized_ads` toggles must not return. |
| `apps/app-frontend/src/components/ui/PromotionWrapper.vue` | **Deleted.** If upstream resurrects it, delete it again and strip new imports. |
| `apps/app/src/api/ads.rs` | A minimal plugin that registers every ad command name upstream ships (currently `init_ads_window`, `show_ads_window`, `hide_ads_window`, `show_ads_consent_overlay`, `hide_ads_consent_overlay`, `record_ads_click`, `open_link`, `get_ads_personalization`) with empty `Ok(())` bodies. No `AD_LINK`, no webview construction, no refresh loop. If upstream adds new ad commands, register them here as no-ops too. |
| `apps/app/src/api/ads_occlusion_macos.rs` / `apps/app/src/api/ads_occlusion_windows.rs` | **Deleted.** These were only called by the original ad-webview code and became dead code once `ads.rs` was gutted. If upstream resurrects them, delete them again and re-strip their deps from `apps/app/Cargo.toml` and the workspace `Cargo.toml` (see below). |
| `apps/app/src/api/mod.rs` | No `mod ads_occlusion_macos;` or `mod ads_occlusion_windows;` declarations. |
| `apps/app/Cargo.toml` | No `[target.'cfg(target_os = "macos")'.dependencies]` block (`core-foundation`, `core-graphics`, `objc2-app-kit` were only used by the deleted occlusion module). Windows deps list drops `webview2-com` and `windows-core`. Windows `features` list keeps only `Win32_Foundation`, `Win32_System_Com`, and `Win32_UI_Shell` (the rest were occlusion-only). |
| `Cargo.toml` (workspace) | `webview2-com`, `core-foundation`, `core-graphics`, and `objc2-app-kit` are removed from `[workspace.dependencies]`. Left in place: `windows` and `windows-core` (still used by `packages/app-lib`). |
| `apps/app/tauri.conf.json` | CSP `connect-src` / `font-src` / `script-src` / `frame-src` do not include posthog, sentry, intercom, or tally hosts. |
| `packages/app-lib/src/api/profile/mod.rs` | `analytics/minecraft-server-play` POST and Mojang join-server probe are gone. `try_update_playtime` only folds local playtime into the submitted total — no `analytics/playtime` POST. |
| `packages/app-lib/src/util/fetch.rs` | `download_meta_header` is always `None` so the `modrinth-download-meta` header is never attached. `DownloadMeta` itself can stay defined — only the outgoing header is dropped. |
| `packages/app-lib/src/state/settings.rs` | `Settings::get` returns `telemetry: false` and `personalized_ads: false` regardless of what the row contains. |

## Migrate from Modrinth App

On first launch, if a Modrinth App data directory exists at the platform's standard location and Meverinth's own data directory has not been initialized yet (no `app.db` present), the user is offered a one-time copy of the entire Modrinth App data folder into Meverinth's folder. The original Modrinth App data is left untouched so both apps can run side by side.

| File | Intent to preserve |
| --- | --- |
| `packages/app-lib/src/api/migration.rs` | **New module.** Exposes `find_modrinth_install_candidate(meverinth_identifier)` (returns `Option<MigrationCandidate>` with `source_path` + `estimated_size_bytes`) and `migrate_from_path(meverinth_identifier, source_path)`. Uses `app.db` as the state-marker file: refuses to offer migration if Meverinth already has one, refuses to overwrite if it appears after the offer. Symlinks are intentionally skipped during the copy. |
| `packages/app-lib/src/api/mod.rs` | Declares `pub mod migration;`. |
| `apps/app/src/api/migration.rs` | **New Tauri plugin.** Registers `find_modrinth_install_candidate` and `migrate_from_modrinth` commands that pass `app.config().identifier` through to the app-lib functions. `MigrationCandidate` serializes as camelCase for the frontend. |
| `apps/app/src/api/mod.rs` | Declares `pub mod migration;`. |
| `apps/app/src/main.rs` | Registers `api::migration::init()` in the Tauri builder plugin chain. |
| `apps/app/build.rs` | Inlined plugin `"migration"` with commands `find_modrinth_install_candidate` and `migrate_from_modrinth`, `DefaultPermissionRule::AllowAllCommands`. |
| `apps/app/capabilities/plugins.json` | Lists `"migration:default"` in the permissions array. |
| `apps/app-frontend/src/helpers/migration.ts` | **New helper.** Exports `find_modrinth_install_candidate()` and `migrate_from_modrinth(sourcePath)` plus the `MigrationCandidate` type. |
| `apps/app-frontend/src/components/ui/ModrinthMigrationModal.vue` | **New modal.** Shows source path + estimated size, "Migrate" runs the copy and emits `done(true)` on success, "Start fresh" emits `done(false)`. Header is not closable while busy. |
| `apps/app-frontend/src/App.vue` | The boot flow is now `bootApp()` (replacing the chained `initialize_state().then(setupApp)`). It calls `find_modrinth_install_candidate()` first; if a candidate exists, it sets `migrationCandidate.value`, waits a tick for the modal to mount, opens it, and awaits the user's decision (via `migrationResolver` / `awaitMigrationDecision`) **before** `initialize_state()`. The modal is conditionally rendered via `v-if="migrationCandidate"` and is wired to `handleMigrationDone`. |

**Why the order matters.** `initialize_state()` opens the SQLite database in Meverinth's data dir, so the migration copy has to land in that dir before state init runs — otherwise the live DB would clobber the copied one (or vice versa). If you refactor the boot flow during a rebase, preserve the invariant: migration runs first, state init runs second.

## Offline Minecraft accounts

Lets the user pick between a Microsoft account (existing OAuth flow) and an offline account where they enter a username directly. The launcher then runs Minecraft with that username and a vanilla-style offline UUID.

| File | Intent to preserve |
| --- | --- |
| `Cargo.toml` | `md-5` is in `[workspace.dependencies]`. |
| `packages/app-lib/Cargo.toml` | `md-5 = { workspace = true }` is a direct dep. |
| `packages/app-lib/src/state/minecraft_auth.rs` | Imports `md5::Md5`. Defines `OFFLINE_REFRESH_TOKEN_MARKER` (empty string), `OFFLINE_ACCESS_TOKEN_PLACEHOLDER` (`"0"`), `offline_uuid_for_username` (MD5 v3 over `OfflinePlayer:<name>`), `validate_offline_username` (1–16 chars, `[A-Za-z0-9_]`), `Credentials::new_offline`, `Credentials::is_offline`. **`refresh()` and `online_profile()` must short-circuit when `is_offline()` is true** — those guards are the load-bearing part. |
| `packages/app-lib/src/api/minecraft_auth.rs` | `create_offline_account(username)` validates and upserts the credential as active. |
| `apps/app/src/api/auth.rs` | `login_offline(username) -> Result<Credentials>` Tauri command, registered in the plugin's `generate_handler![…]`. |
| `apps/app/build.rs` | `"login_offline"` is in the `auth` plugin's `.commands(&[…])`. Capability checks fail if this is missing. |
| `apps/app-frontend/src/helpers/auth.js` | `login_offline(username)` helper that invokes `plugin:auth\|login_offline`. |
| `apps/app-frontend/src/components/ui/AddAccountModal.vue` | **New file.** Two-step picker: choose Microsoft (runs the existing OAuth flow) or Offline (shows a username form with live validation). Emits `added` with a credential the parent passes to `setAccount`. |
| `apps/app-frontend/src/components/ui/AccountsCard.vue` | "Sign in to Minecraft" / "Add account" buttons open `AddAccountModal` via `openAddAccount()`. The component no longer imports `login` from `@/helpers/auth` directly — the modal owns the Microsoft flow now. |

## Launch path dependencies

The launcher's argument substitution (`packages/app-lib/src/launcher/args.rs`) reads `credentials.access_token`, `profile.name`, and `profile.id`. `Credentials::new_offline` populates all three with sensible values (placeholder token, entered username, derived UUID). If upstream changes how launch arguments are built, make sure offline credentials still produce a launchable command — the username and UUID values must still flow into `${auth_player_name}` / `${auth_uuid}` / `${uuid}`.

## Signals of regression

If any of these appear after the rebase, something slipped through:

- A new `import { posthog }`, `import * as Sentry`, `import { Intercom }`, or `import.*'@intercom/messenger-js-sdk'` anywhere in `apps/app-frontend/src`.
- A new third-party `<script src="https://…">` in `apps/app-frontend/index.html`.
- A new `fetch::post_json(.., "analytics/…")` in `packages/app-lib/src/api/`.
- A new CSP entry in `apps/app/tauri.conf.json` referencing a tracker host.
- `Credentials::new_offline` failing to compile because `Credentials` gained a new required field — add the field with a sensible offline default.
- `AccountsCard.vue` directly calling `login_flow()` again — route it back through `AddAccountModal`.

## Adding a new entry

When you make a new fork-specific change that the rebase must preserve, append a row to the most fitting capability section (or add a new section if it's a new capability). Each row should answer two questions: **which file** and **what behavior survives**. Don't describe the diff — describe the invariant.
