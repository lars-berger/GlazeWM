use anyhow::Context;
use wm_common::{TilingDirection, WmEvent, WorkspaceConfig};

use super::sort_workspaces;
use crate::{
  containers::{
    commands::attach_container,
    traits::{CommonGetters, PositionGetters},
  },
  monitors::Monitor,
  user_config::UserConfig,
  wm_state::WmState,
  workspaces::Workspace,
};

/// Activates a workspace on the target monitor.
///
/// If no workspace name is provided, the first suitable workspace defined
/// in the user's config will be used.
///
/// If no target monitor is provided, the workspace is activated on
/// whichever monitor it is bound to, or the currently focused monitor.
pub fn activate_workspace(
  workspace_name: Option<&str>,
  target_monitor: Option<Monitor>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace_config = workspace_config(
    workspace_name,
    target_monitor.clone(),
    state,
    config,
  )?;

  let target_monitor = target_monitor
    .or_else(|| {
      workspace_config
        .bind_to_monitor
        .and_then(|index| {
          state
            .monitors()
            .into_iter()
            .find(|monitor| monitor.index() == index as usize)
        })
        .or_else(|| {
          state
            .focused_container()
            .and_then(|focused| focused.monitor())
        })
    })
    .context("Failed to get a target monitor for the workspace.")?;

  let monitor_rect = target_monitor.to_rect()?;
  let tiling_direction = match monitor_rect.height() > monitor_rect.width()
  {
    true => TilingDirection::Vertical,
    false => TilingDirection::Horizontal,
  };

  let workspace = Workspace::new(
    workspace_config.clone(),
    config.value.gaps.clone(),
    tiling_direction,
  );

  // Attach the created workspace to the specified monitor.
  attach_container(
    &workspace.clone().into(),
    &target_monitor.clone().into(),
    None,
  )?;

  sort_workspaces(target_monitor.clone(), config)?;

  state.emit_event(WmEvent::WorkspaceActivated {
    activated_workspace: workspace.to_dto()?,
  });

  Ok(())
}

/// Gets config for the workspace to activate.
fn workspace_config(
  workspace_name: Option<&str>,
  target_monitor: Option<Monitor>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WorkspaceConfig> {
  let found_config = match workspace_name {
    Some(workspace_name) => config
      .inactive_workspace_configs(&state.workspaces())
      .into_iter()
      .find(|config| config.name == workspace_name)
      .with_context(|| {
        format!(
          "Workspace with name '{}' doesn't exist or is already active.",
          workspace_name
        )
      }),
    None => target_monitor
      .and_then(|target_monitor| {
        config.workspace_config_for_monitor(
          &target_monitor,
          &state.workspaces(),
        )
      })
      .or_else(|| {
        config.next_inactive_workspace_config(&state.workspaces())
      })
      .context("No workspace config available to activate workspace."),
  };

  found_config.cloned()
}
