<h1 align="center">
    Bevy Easy Websockets
</h1>

<p align="center">
    <a href="https://crates.io/crates/bevy_easy_websockets">
        <img src="https://img.shields.io/crates/v/bevy_easy_websockets?logo=rust" alt="crates.io">
    </a>
    <a href="https://docs.rs/bevy_easy_websockets">
        <img src="https://docs.rs/bevy_easy_websockets/badge.svg" alt="docs.rs">
    </a>
</p>

> `bevy_easy_websockets` is websocket based networking solution for [bevy](https://bevyengine.org/) games. It is forked from the Bevy_spicy_networking and would not be here if not for that. Thank you!

<hr>

Using tokio as the asynchronous backend along with tungstenite for the websocket implemention, it fully integrates into bevy to allow you to quickly add networking to your game.
Simply add either the server/client plugins, register the types of messages you plan on receiving and listen for them as events!

Bevy_easy_websockets is intended to support both games and other crates, network messages are generic and are extensible by other downstream crates of whatever library uses bevy_easy_websockets. This is achieved through the amazing [`typetag`](https://github.com/dtolnay/typetag) crate.

<hr>

### Contents
- [Documentation](#documentation)
  - [Quickstart](#quickstart)
- [Bevy Version Compatibility](#bevy-version-compatibility)
- [Supported Platforms](#supported-platforms)
- [Roadmap](#roadmap)
- [Crates using `bevy_simple_websockets`](#crates-using-bevy_easy_websockets)
- [Contributing](#contributing)


Documentation
-------------

You can check out the [**online documentation**](https://docs.rs/bevy_easy_websockets), or build it yourself by cloning this repo and running `cargo doc -p bevy_easy_websockets`.

For examples, check out the [examples directory](https://github.com/NoahShomette/bevy_easy_websockets/tree/main/examples).

- In `server.rs` you will find a simple chat server, that broadcasts the messages it receives from clients
- In `client.rs` you will find a simple graphical chat client, where you can connect to a server and send messages to
- IN 'shared.rs' you will find all the shared logic and messages for both clients

(Note: Since bevy does not include a text input widget, it is a very simplified demo. This should be easy to extend once the UI part of bevy 
is more complete.)

### Quickstart

1. Add `bevy_easy_websockets`, `serde_derive` and `typetag` to your `Cargo.toml`
2. Create the messages you wish to exchange beetween a client and server, or vice-versa.
    - Implement Serialize and Deserialize from Serde on it
    - Implement `NetworkMessage`, and make sure to annotate it with `typetag::serde`
    - Implement `ServerMessage` when it is sent to the server from a client
    - Implement `ClientMessage` when it is sent to a client from the server

```rust
#[derive(Serialize, Deserialize)]
struct WhisperMessage {
    recipient: UserId,
    message: String,
}

#[typetag::serde]
impl NetworkMessage for WhisperMessage {}

// In this case, its a client sending a message to a server
impl ServerMessage for WhisperMessage {
    const NAME: &'static str = "example:WhisperMessage"; // This name needs to be unique!
    // Otherwise, the recipient will mix up the messages
}
```

3. On the recipient side, register the type to be received

```rust
use bevy_simple_websockets::AppNetworkServerMessage;

let appbuilder: &mut AppBuilder = /* Get an AppBuilder, which is returned by bevy from App::build() */;


// Now whenever a client sends a `WhisperMessage` the server will generate an event of
// `NetworkData<WhisperMessage>` which your application can then handle
appbuilder.listen_for_server_message::<WhisperMessage>();
```

4. Listen for events of that type

```rust
fn handle_incoming_whisper_messages(
    mut whisper_messages: EventReader<NetworkMessage<WhisperMessage>>,
) {
    for whisper_message in whisper_messages.iter() {
        // Handle the whisper
    }
}
```


Bevy Version Compatibility
--------------------------

Simply pick the version compatible to your bevy version:

| Bevy | bevy_easy_websockets  |
| :-------------------: | :---: |
|          0.9          |  0.1  |

Any version that is not compatible with the latest bevy version is in maintenance mode.
It will only receive minor bug fixes from my side, or community supplied ones.

Supported Platforms
-------------------

**Windows** is supported officially and works. WASM is a goal of the project however it does not work currently. It should work on all other dedicated platforms though I do not have the capabilities to test Mac or Linux. If it doesn't work on either of those open an issue and help me figure it out!


Crates using `bevy_easy_websockets`
--------------------

> Currently none, you can help by expanding this list. Just send a PR and add it to the table below!

| Name | Version |
|:----:|:-------:|
|  -   |    -    |

Contributing
------------

To contribute, simply fork the repository and send a PR. Feel free to chat me up on the bevy discord under `@NickHaldon#3854` if you have any questions or suggestions.
