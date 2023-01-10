#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    clippy::unwrap_used
)]
#![allow(clippy::type_complexity)]

/*!
A simple websocket based networking plugin for Bevy

This library was forked from [Bevy_Spicy_Networking](https://github.com/CabbitStudios/bevy_spicy_networking).
This library would not be here if it wasn't for the excellent work done by CabbitStudios originally. Thank you!

Using this plugin is meant to be straightforward. You have one server and multiple clients.
You simply add either the `ClientPlugin` or the `ServerPlugin` to the respective bevy app,
register which kind of messages can be received through `listen_for_client_message` or `listen_for_server_message`
(provided respectively by `AppNetworkClientMessage` and `AppNetworkServerMessage`) and you
can start receiving packets as events of `NetworkData<T>`.

## Example Client
```rust,no_run
use bevy::prelude::*;
use bevy_simple_websockets::{ClientPlugin, NetworkData, NetworkMessage, ServerMessage, ClientNetworkEvent, AppNetworkServerMessage};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct WorldUpdate;

#[typetag::serde]
impl NetworkMessage for WorldUpdate {}

#[typetag::serde]
impl ServerMessage for WorldUpdate {
    const NAME: &'static str = "example:WorldUpdate";
}

fn main() {
     let mut app = App::build();
     app.add_plugin(ClientPlugin);
     // We are receiving this from the server, so we need to listen for it
     app.listen_for_server_message::<WorldUpdate>();
     app.add_system(handle_world_updates);
     app.add_system(handle_connection_events);
}

fn handle_world_updates(
    mut chunk_updates: EventReader<NetworkData<WorldUpdate>>,
) {
    for chunk in chunk_updates.iter() {
        info!("Got chunk update!");
    }
}

fn handle_connection_events(mut network_events: EventReader<ClientNetworkEvent>,) {
    for event in network_events.iter() {
        match event {
            &ClientNetworkEvent::Connected => info!("Connected to server!"),
            _ => (),
        }
    }
}

```

## Example Server
```rust,no_run
use bevy::prelude::*;
use bevy_simple_websockets::{ServerPlugin, NetworkData, NetworkMessage, NetworkServer, ServerMessage, ClientMessage, ServerNetworkEvent, AppNetworkClientMessage};

use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize)]
struct UserInput;

#[typetag::serde]
impl NetworkMessage for UserInput {}

//#[typetag::serde]
impl ClientMessage for UserInput {
    const NAME: &'static str = "example:UserInput";
}

fn main() {
     let mut app = App::build();
     app.add_plugin(ServerPlugin);
     // We are receiving this from a client, so we need to listen for it!
     app.listen_for_client_message::<UserInput>();
     app.add_system(handle_world_updates.system());
     app.add_system(handle_connection_events.system());
}

fn handle_world_updates(
    net: Res<NetworkServer>,
    mut chunk_updates: EventReader<NetworkData<UserInput>>,
) {
    for chunk in chunk_updates.iter() {
        info!("Got chunk update!");
    }
}

#[derive(Serialize, Deserialize)]
struct PlayerUpdate;

#[typetag::serde]
impl NetworkMessage for PlayerUpdate {}

impl ClientMessage for PlayerUpdate {
    const NAME: &'static str = "example:PlayerUpdate";
}

impl PlayerUpdate {
    fn new() -> PlayerUpdate {
        Self
    }
}

fn handle_connection_events(
    net: Res<NetworkServer>,
    mut network_events: EventReader<ServerNetworkEvent>,
) {
    for event in network_events.iter() {
        match event {
            &ServerNetworkEvent::Connected(conn_id) => {
                let connection_result = net.send_message(conn_id, PlayerUpdate::new());
                info!("New client connected: {:?}", conn_id);
            }
            _ => (),
        }
    }
}

```
As you can see, they are both quite similar, and provide everything a basic networked game needs.

For a more

## Caveats

This library is built using WebSockets and thus TCP at the core. It is reliable and ordered meaning
large packets can block the connection while they are transmitted. This is generally not suitable for
fast paced or competitive games. It should work for slower paced games or games where latency delays
are acceptable

*/

mod client;
mod error;
mod network_message;
mod server;

use bevy::{prelude::*, utils::Uuid};
pub use client::{AppNetworkClientMessage, NetworkClient};
use crossbeam_channel::{unbounded, Receiver, Sender};
use derive_more::{Deref, Display};
use error::NetworkError;
pub use network_message::{ClientMessage, NetworkMessage, ServerMessage};
use serde::{Deserialize, Serialize};
pub use server::{AppNetworkServerMessage, NetworkServer};
use url::{Host, Url};

struct SyncChannel<T> {
    pub(crate) sender: Sender<T>,
    pub(crate) receiver: Receiver<T>,
}

impl<T> SyncChannel<T> {
    fn new() -> Self {
        let (sender, receiver) = unbounded();

        SyncChannel { sender, receiver }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Display, Debug)]
#[display(fmt = "Connection from {} with ID={}", url, uuid)]
/// A [`ConnectionId`] denotes a single connection
///
/// Use [`ConnectionId::is_server`] to check whether it is a connection to a server
/// or another. In most client/server applications this is not required as there
/// is no ambiguity.
pub struct ConnectionId {
    uuid: Uuid,
    url: Url,
}

impl ConnectionId {
    /// Get the address associated to this connection id
    ///
    /// This contains the IP/Port information
    pub fn address(&self) -> Option<Host<&str>> {
        self.url.host()
    }

    pub(crate) fn server(addr: Option<Url>) -> ConnectionId {
        ConnectionId {
            uuid: Uuid::nil(),
            url: addr.unwrap_or_else(|| Url::parse("ws://127.0.0.1:9999").unwrap()),
        }
    }

    /// Check whether this [`ConnectionId`] is a server
    pub fn is_server(&self) -> bool {
        self.uuid == Uuid::nil()
    }
}

#[derive(Serialize, Deserialize)]
/// [`NetworkPacket`]s are untyped packets to be sent over the wire
struct NetworkPacket {
    kind: String,
    data: Box<dyn NetworkMessage>,
}

impl std::fmt::Debug for NetworkPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkPacket")
            .field("kind", &self.kind)
            .finish()
    }
}

/// A network event originating from a [`NetworkServer`]
#[derive(Debug)]
pub enum ServerNetworkEvent {
    /// A new client has connected
    Connected(ConnectionId),
    /// A client has disconnected
    Disconnected(ConnectionId),
    /// An error occured while trying to do a network operation
    Error(NetworkError),
}

#[derive(Debug)]
/// A network event originating from a [`NetworkClient`]
pub enum ClientNetworkEvent {
    /// Connected to a server
    Connected,
    /// Disconnected from a server
    Disconnected,
    /// An error occured while trying to do a network operation
    Error(NetworkError),
}

#[derive(Debug, Deref)]
/// [`NetworkData`] is what is sent over the bevy event system
///
/// Please check the root documentation for how to set up everything
pub struct NetworkData<T> {
    source: ConnectionId,
    #[deref]
    inner: T,
}

impl<T> NetworkData<T> {
    pub(crate) fn new(source: ConnectionId, inner: T) -> Self {
        Self { source, inner }
    }

    /// The source of this network data
    pub fn source(&self) -> ConnectionId {
        self.source.clone()
    }

    /// Get the inner data out of it
    pub fn into_inner(self) -> T {
        self.inner
    }
}

#[derive(Clone, Debug, Resource)]
#[allow(missing_copy_implementations)]
/// Settings to configure the network, both client and server
pub struct NetworkSettings {
    /// Maximum packet size in bytes. If a client ever exceeds this size, they will be disconnected
    ///
    /// ## Default
    /// The default is set to 10MiB
    pub max_packet_length: usize,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        NetworkSettings {
            max_packet_length: 10 * 1024 * 1024,
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
/// The plugin to add to your bevy [`App`](App) when you want
/// to instantiate a server
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkServer::new());
        app.add_event::<ServerNetworkEvent>();
        app.init_resource::<NetworkSettings>();
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            server::handle_new_incoming_connections,
        );
    }
}

#[derive(Default, Copy, Clone, Debug)]
/// The plugin to add to your bevy [`App`](App) when you want
/// to instantiate a client
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkClient::new());
        app.add_event::<ClientNetworkEvent>();
        app.init_resource::<NetworkSettings>();
        app.add_system_to_stage(CoreStage::PreUpdate, client::send_client_network_events);
        app.add_system_to_stage(CoreStage::PreUpdate, client::handle_connection_event);
    }
}
