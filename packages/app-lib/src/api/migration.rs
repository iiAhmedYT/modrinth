//! One-time data migration from the upstream Modrinth App into Meverinth.
//!
//! Meverinth uses its own app identifier (and therefore its own data
//! directory) so it can sit side-by-side with the original Modrinth App. To
//! keep existing users from losing their instances, accounts, and settings
//! when they switch over, we let the frontend offer a one-time copy from the
//! Modrinth App data directory the first time Meverinth is launched.

use crate::ErrorKind;
use crate::state::DirectoryInfo;
use async_walkdir::WalkDir;
use futures::StreamExt;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Data directory name used by the upstream Modrinth App. Meverinth never
/// reads or writes inside this directory other than to copy from it.
const MODRINTH_APP_IDENTIFIER: &str = "ModrinthApp";

/// File whose presence in a settings directory means the directory has
/// already been used by either the Modrinth App or Meverinth, and copying on
/// top of it would clobber live state.
const STATE_MARKER_FILE: &str = "app.db";

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrationCandidate {
    /// Absolute path to the existing Modrinth App data directory.
    pub source_path: PathBuf,
    /// Best-effort sum of file sizes inside the source directory.
    pub estimated_size_bytes: u64,
}

/// Returns `Some` when there is a Modrinth App data directory on disk that
/// looks like a real install, and our own data directory has not yet been
/// initialized (so a migration won't clobber anything).
#[tracing::instrument]
pub async fn find_modrinth_install_candidate(
    meverinth_identifier: &str,
) -> crate::Result<Option<MigrationCandidate>> {
    if let Some(our_dir) =
        DirectoryInfo::initial_settings_dir_path(meverinth_identifier)
        && has_state_marker(&our_dir)
    {
        return Ok(None);
    }

    let modrinth_dir = match dirs::data_dir() {
        Some(base) => base.join(MODRINTH_APP_IDENTIFIER),
        None => return Ok(None),
    };

    if !modrinth_dir.is_dir() || !has_state_marker(&modrinth_dir) {
        return Ok(None);
    }

    let estimated_size_bytes =
        dir_size_bytes(&modrinth_dir).await.unwrap_or(0);

    Ok(Some(MigrationCandidate {
        source_path: modrinth_dir,
        estimated_size_bytes,
    }))
}

/// Copies the Modrinth App data directory at `source_path` into Meverinth's
/// data directory. Refuses to run if Meverinth's directory already contains a
/// state marker, so an accidental re-trigger can't overwrite a live install.
#[tracing::instrument]
pub async fn migrate_from_path(
    meverinth_identifier: &str,
    source_path: PathBuf,
) -> crate::Result<()> {
    let dest_dir =
        DirectoryInfo::initial_settings_dir_path(meverinth_identifier)
            .ok_or_else(|| {
                ErrorKind::FSError(
                    "Could not resolve Meverinth data directory".to_string(),
                )
            })?;

    if has_state_marker(&dest_dir) {
        return Err(ErrorKind::OtherError(
            "Meverinth has already been initialized; refusing to overwrite \
             its data with a migration"
                .to_string(),
        )
        .into());
    }

    if !source_path.is_dir() {
        return Err(ErrorKind::FSError(format!(
            "Source path {} is not a directory",
            source_path.display()
        ))
        .into());
    }

    fs::create_dir_all(&dest_dir).await.map_err(|err| {
        ErrorKind::FSError(format!(
            "Failed to create Meverinth data directory: {err}"
        ))
    })?;

    copy_dir_recursive(&source_path, &dest_dir).await
}

fn has_state_marker(dir: &Path) -> bool {
    dir.join(STATE_MARKER_FILE).is_file()
}

async fn copy_dir_recursive(src: &Path, dst: &Path) -> crate::Result<()> {
    let mut walker = WalkDir::new(src);
    while let Some(entry) = walker.next().await {
        let entry = entry.map_err(|err| {
            ErrorKind::FSError(format!(
                "Failed to read entry under {}: {err}",
                src.display()
            ))
        })?;

        let file_type = entry.file_type().await.map_err(|err| {
            ErrorKind::FSError(format!(
                "Failed to read file type for {}: {err}",
                entry.path().display()
            ))
        })?;

        // Symlinks are intentionally skipped: blindly resolving them could
        // copy in data from outside the source directory, and recreating the
        // symlink target wouldn't necessarily be valid after the copy.
        if file_type.is_symlink() {
            tracing::debug!(
                "Skipping symlink during migration: {}",
                entry.path().display()
            );
            continue;
        }

        let entry_path = entry.path();
        let relative = entry_path.strip_prefix(src).map_err(|err| {
            ErrorKind::FSError(format!(
                "Failed to relativize migration path {}: {err}",
                entry_path.display()
            ))
        })?;
        let target = dst.join(relative);

        if file_type.is_dir() {
            fs::create_dir_all(&target).await.map_err(|err| {
                ErrorKind::FSError(format!(
                    "Failed to create directory {}: {err}",
                    target.display()
                ))
            })?;
            continue;
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).await.map_err(|err| {
                ErrorKind::FSError(format!(
                    "Failed to create directory {}: {err}",
                    parent.display()
                ))
            })?;
        }

        fs::copy(entry_path.as_path(), &target).await.map_err(|err| {
            ErrorKind::FSError(format!(
                "Failed to copy {} to {}: {err}",
                entry_path.display(),
                target.display()
            ))
        })?;
    }
    Ok(())
}

async fn dir_size_bytes(dir: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    let mut walker = WalkDir::new(dir);
    while let Some(entry) = walker.next().await {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type().await else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        if let Ok(meta) = entry.metadata().await {
            total = total.saturating_add(meta.len());
        }
    }
    Ok(total)
}
