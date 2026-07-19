# ![Meverinth](/.github/assets/app_cover.png)

## Meverinth

Meverinth is a privacy-focused fork of the [Modrinth App](https://modrinth.com/app), a desktop application for managing your Minecraft mods. It is built with [Tauri](https://tauri.app/) and [Vue](https://vuejs.org/).

Compared to upstream, Meverinth has telemetry, error reporting, the in-app support chat, user surveys, ads, and download/playtime analytics removed, and adds support for offline-mode Minecraft accounts alongside the existing Microsoft sign-in flow.

## Development

### Pre-requisites

Before you begin, ensure you have the following installed on your machine:

- [Node.js](https://nodejs.org/en/)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri](https://v2.tauri.app/start/prerequisites/)

### Setup

Follow these steps to set up your development environment:

```bash
pnpm install
pnpm app:dev
```

You should now have a development build of the app running with hot-reloading enabled. Any changes you make to the code will automatically refresh the app.
