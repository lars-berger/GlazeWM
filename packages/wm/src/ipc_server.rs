use std::{iter, net::SocketAddr};

use anyhow::Context;
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::{
  net::{TcpListener, TcpStream},
  sync::{broadcast, mpsc},
  task,
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
  app_command::{AppCommand, QueryCommand, SubscribableEvent},
  containers::{Container, WindowContainer},
  monitors::Monitor,
  user_config::UserConfig,
  wm::WindowManager,
  wm_event::WmEvent,
  workspaces::Workspace,
};

pub const DEFAULT_IPC_PORT: u32 = 6123;

#[derive(Debug, Serialize)]
#[serde(tag = "message_type", rename_all = "snake_case")]
pub enum ServerMessage {
  ClientResponse(ClientResponseMessage),
  EventSubscription(EventSubscriptionMessage),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientResponseMessage {
  client_message: String,
  data: Option<ClientResponseData>,
  error: Option<String>,
  success: bool,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ClientResponseData {
  BindingModes(Vec<String>),
  Command(CommandData),
  EventSubscription(EventSubscriptionData),
  Focused(Option<Container>),
  Monitors(Vec<Monitor>),
  Windows(Vec<WindowContainer>),
  Workspaces(Vec<Workspace>),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandData {
  subject_container_id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSubscriptionData {
  subscription_id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSubscriptionMessage {
  data: Option<serde_json::Value>,
  error: Option<String>,
  subscription_id: Uuid,
  success: bool,
}

pub struct IpcServer {
  abort_handle: task::AbortHandle,
  pub message_rx: mpsc::UnboundedReceiver<(
    String,
    mpsc::UnboundedSender<Message>,
    broadcast::Sender<()>,
  )>,
  event_rx: broadcast::Receiver<(SubscribableEvent, serde_json::Value)>,
  event_tx: broadcast::Sender<(SubscribableEvent, serde_json::Value)>,
}

impl IpcServer {
  pub async fn start() -> anyhow::Result<Self> {
    let (message_tx, message_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = broadcast::channel(16);

    let server_addr = format!("127.0.0.1:{}", DEFAULT_IPC_PORT);
    let server = TcpListener::bind(server_addr.clone()).await?;
    info!("IPC server started on: '{}'.", server_addr);

    let task = task::spawn(async move {
      while let Ok((stream, addr)) = server.accept().await {
        let message_tx = message_tx.clone();

        task::spawn(async move {
          if let Err(err) =
            Self::handle_connection(stream, addr, message_tx).await
          {
            error!("Error handling connection: {}", err);
          }
        });
      }
    });

    Ok(Self {
      message_rx,
      event_rx,
      event_tx,
      abort_handle: task.abort_handle(),
    })
  }

  async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    message_tx: mpsc::UnboundedSender<(
      String,
      mpsc::UnboundedSender<Message>,
      broadcast::Sender<()>,
    )>,
  ) -> anyhow::Result<()> {
    info!("Incoming IPC connection from: {}.", addr);

    let ws_stream = accept_async(stream)
      .await
      .context("Error during websocket handshake.")?;

    let (mut outgoing, mut incoming) = ws_stream.split();
    let (response_tx, mut response_rx) = mpsc::unbounded_channel();
    let (disconnection_tx, _) = broadcast::channel(16);

    loop {
      tokio::select! {
        Some(response) = response_rx.recv() => {
          if let Err(err) = outgoing.send(response).await {
            error!("Error sending response: {}", err);
            break;
          }
        }
        message = incoming.next() => {
          if let Some(Ok(message)) = message {
            if message.is_text() || message.is_binary() {
              message_tx.send((
                message.to_text()?.to_owned(),
                response_tx.clone(),
                disconnection_tx.clone(),
              ))?;
            }
          } else {
            warn!("Could not read next websocket message.");
            break;
          }
        }
      }
    }

    info!("IPC disconnection from: {}.", addr);
    disconnection_tx.send(())?;

    Ok(())
  }

  pub fn process_message(
    &self,
    message: String,
    response_tx: mpsc::UnboundedSender<Message>,
    disconnection_tx: broadcast::Sender<()>,
    wm: &mut WindowManager,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let app_command = AppCommand::try_parse_from(
      iter::once("").chain(message.split_whitespace()),
    );

    let response = match app_command {
      Ok(AppCommand::Query { command }) => match command {
        QueryCommand::Windows => {
          Ok(ClientResponseData::Windows(wm.state.windows()))
        }
        QueryCommand::Workspaces => {
          Ok(ClientResponseData::Workspaces(wm.state.workspaces()))
        }
        QueryCommand::Monitors => {
          Ok(ClientResponseData::Monitors(wm.state.monitors()))
        }
        QueryCommand::BindingModes => Ok(
          ClientResponseData::BindingModes(wm.state.binding_modes.clone()),
        ),
        QueryCommand::Focused => {
          Ok(ClientResponseData::Focused(wm.state.focused_container()))
        }
      },
      Ok(AppCommand::Cmd {
        subject_container_id,
        command,
      }) => wm
        .process_commands(vec![command], subject_container_id, config)
        .map(|subject_container_id| {
          ClientResponseData::Command(CommandData {
            subject_container_id,
          })
        }),
      Ok(AppCommand::Subscribe { events }) => {
        let subscription_id = Uuid::new_v4();
        let response_tx = response_tx.clone();
        let mut event_rx = self.event_tx.subscribe();
        let mut disconnection_rx = disconnection_tx.subscribe();

        task::spawn(async move {
          loop {
            tokio::select! {
              // TODO: Also listen to unsubscribe messages.
              Ok(_) = disconnection_rx.recv() => {
                break;
              }
              Ok((event, event_json)) = event_rx.recv() => {
                let message = EventSubscriptionMessage {
                  data: Some(event_json),
                  error: None,
                  subscription_id: subscription_id.clone(),
                  success: true,
                };

                let response_msg = Message::Text(serde_json::to_string(&message).unwrap());
                response_tx.send(response_msg).unwrap();
              }
            }
          }
        });

        Ok(ClientResponseData::EventSubscription(
          EventSubscriptionData { subscription_id },
        ))
      }
      Err(err) => Err(anyhow::anyhow!(err)),
      _ => Err(anyhow::anyhow!("Unsupported IPC command.")),
    };

    let error = response.as_ref().err().map(|err| err.to_string());
    let success = response.as_ref().is_ok();

    let response = ClientResponseMessage {
      client_message: message,
      data: response.ok(),
      error,
      success,
    };

    // Respond to the client with the result of the command.
    let response_msg = Message::Text(serde_json::to_string(&response)?);
    response_tx.send(response_msg)?;

    Ok(())
  }

  pub fn process_event(&mut self, event: WmEvent) -> anyhow::Result<()> {
    let subscribable_event = match event {
      WmEvent::BindingModesChanged { .. } => {
        SubscribableEvent::BindingModesChanged
      }
      WmEvent::FocusChanged { .. } => SubscribableEvent::FocusChanged,
      WmEvent::FocusedContainerMoved { .. } => {
        SubscribableEvent::FocusedContainerMoved
      }
      WmEvent::MonitorAdded { .. } => SubscribableEvent::MonitorAdded,
      WmEvent::MonitorUpdated { .. } => SubscribableEvent::MonitorUpdated,
      WmEvent::MonitorRemoved { .. } => SubscribableEvent::MonitorRemoved,
      WmEvent::TilingDirectionChanged { .. } => {
        SubscribableEvent::TilingDirectionChanged
      }
      WmEvent::UserConfigChanged { .. } => {
        SubscribableEvent::UserConfigChanged
      }
      WmEvent::WindowManaged { .. } => SubscribableEvent::WindowManaged,
      WmEvent::WindowUnmanaged { .. } => {
        SubscribableEvent::WindowUnmanaged
      }
      WmEvent::WorkspaceActivated { .. } => {
        SubscribableEvent::WorkspaceActivated
      }
      WmEvent::WorkspaceDeactivated { .. } => {
        SubscribableEvent::WorkspaceDeactivated
      }
      WmEvent::WorkspaceMoved { .. } => SubscribableEvent::WorkspaceMoved,
    };

    let event_json = serde_json::to_value(&event)?;
    self.event_tx.send((subscribable_event, event_json))?;

    Ok(())
  }

  pub fn stop(&self) {
    self.abort_handle.abort();
  }
}

impl Drop for IpcServer {
  fn drop(&mut self) {
    self.stop();
  }
}
