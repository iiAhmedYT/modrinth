use crate::server_address::ServerAddress;
use crate::state::{Credentials, ProcessMetadata, Settings, State};
use crate::util::io::IOError;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub enum QuickPlayType {
    None,
    Singleplayer(String),
    Server(ServerAddress),
}

#[tracing::instrument]
pub async fn run(
    instance_id: &str,
    quick_play_type: QuickPlayType,
) -> crate::Result<ProcessMetadata> {
    let state = State::get().await?;
    let default_account = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or_else(|| crate::ErrorKind::NoCredentialsError.as_error())?;

    run_credentials(instance_id, &default_account, quick_play_type).await
}

#[tracing::instrument(skip(credentials))]
async fn run_credentials(
    instance_id: &str,
    credentials: &Credentials,
    quick_play_type: QuickPlayType,
) -> crate::Result<ProcessMetadata> {
    let state = State::get().await?;
    let settings = Settings::get(&state.pool).await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to run a nonexistent instance {instance_id}!"
            ))
        })?;

    let pre_launch_hooks = context
        .launch_overrides
        .hooks
        .pre_launch
        .as_ref()
        .or(settings.hooks.pre_launch.as_ref())
        .filter(|hook_command| !hook_command.is_empty());
    if let Some(hook) = pre_launch_hooks {
        let mut cmd = shlex::split(hook)
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!(
                    "Invalid pre-launch command: {hook}",
                ))
            })?
            .into_iter();

        if let Some(command) = cmd.next() {
            let full_path = crate::util::io::canonicalize(
                state
                    .directories
                    .instances_dir()
                    .join(&context.instance.path),
            )?;
            let result = Command::new(command)
                .args(cmd)
                .current_dir(&full_path)
                .spawn()
                .map_err(|e| IOError::with_path(e, &full_path))?
                .wait()
                .await
                .map_err(IOError::from)?;

            if !result.success() {
                return Err(crate::ErrorKind::LauncherError(format!(
                    "Non-zero exit code for pre-launch hook: {}",
                    result.code().unwrap_or(-1)
                ))
                .as_error());
            }
        }
    }

    let java_args = context
        .launch_overrides
        .extra_launch_args
        .clone()
        .unwrap_or(settings.extra_launch_args);
    let wrapper = context
        .launch_overrides
        .hooks
        .wrapper
        .clone()
        .or(settings.hooks.wrapper)
        .filter(|hook_command| !hook_command.is_empty());
    let memory = context.launch_overrides.memory.unwrap_or(settings.memory);
    let resolution = context
        .launch_overrides
        .game_resolution
        .unwrap_or(settings.game_resolution);
    let env_args = context
        .launch_overrides
        .custom_env_vars
        .clone()
        .unwrap_or(settings.custom_env_vars);
    let post_exit_hook = context
        .launch_overrides
        .hooks
        .post_exit
        .clone()
        .or(settings.hooks.post_exit)
        .filter(|hook_command| !hook_command.is_empty());

    let mut mc_set_options: Vec<(String, String)> = vec![];
    if let Some(fullscreen) = context.launch_overrides.force_fullscreen {
        mc_set_options.push(("fullscreen".to_string(), fullscreen.to_string()));
    } else if settings.force_fullscreen {
        mc_set_options.push(("fullscreen".to_string(), "true".to_string()));
    }

    // Server-play analytics reporting has been removed from this privacy-focused build.

    crate::minecraft_skins::flush_pending_skin_change().await?;
    crate::launcher::launch_minecraft(
        &java_args,
        &env_args,
        &mc_set_options,
        &wrapper,
        &memory,
        &resolution,
        credentials,
        post_exit_hook,
        &context,
        quick_play_type,
    )
    .await
}

pub async fn kill(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let processes =
        crate::api::process::get_by_instance_id(instance_id).await?;

    for process in processes {
        state.process_manager.kill(process.uuid).await?;
    }

    Ok(())
}

/// Playtime reporting to Modrinth has been removed in this privacy-focused
/// build. Recent playtime is still tracked locally for the UI but is never
/// transmitted; we just fold it into the submitted total and clear it.
#[tracing::instrument]
pub async fn try_update_playtime_by_instance_id(
    instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to update playtime for nonexistent instance {instance_id}!"
            ))
        })?;
    let updated_recent_playtime = context.instance.recent_time_played;

    if updated_recent_playtime > 0 {
        crate::state::instances::commands::mark_instance_playtime_submitted(
            &context.instance.id,
            updated_recent_playtime,
            &state.pool,
        )
        .await?;
    }

    Ok(())
}
