use std::sync::Arc;

use bytes::{BytesMut, Bytes};
use futures::{SinkExt, StreamExt};
use protocol::{FrameType, TibiaCodec};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

pub mod debug;
pub mod login;
pub mod game;

/// Event handler for extending the proxy functionality
/// Proxy event handlers always run in the order they were added to the proxy
pub trait ProxyEventHandler {
    /// Runs when a new client has connected to the proxy
    /// Return an error to disconnect the proxy
    fn on_new_connection(&self, _connection: &mut ProxyConnection) -> anyhow::Result<()> { Ok(()) }

    /// Runs when both the server and client is connected and we are ready to start proxying
    /// Return an error to disconnect the proxy
    fn on_ready(&self, _connection: &mut ProxyConnection) -> anyhow::Result<()> { Ok(()) }

    /// Runs after the server or client disconnects, or if there is an error which results in disconnection
    fn on_disconnect(&self, _connection: &mut ProxyConnection, _reason: &DisconnectReason) { }

    /// Acts as a middleware for each frame.
    /// Return an error to disconnect the proxy
    fn on_frame(&self, _connection: &mut ProxyConnection, _from: Origin, frame: BytesMut) -> anyhow::Result<BytesMut> { Ok(frame) }
}

#[derive(Debug)]
pub enum DisconnectReason {
    DisconnectedBy(Origin),
    Error(anyhow::Error),
}

/// Used to specify the origin of a frame or disconnect
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Origin {
    Client,
    Server,
}

pub struct ProxyBuilder {
    listen_addr: String,
    server_addr: String,
    event_handlers: Vec<Box<dyn ProxyEventHandler + Send + Sync>>,
}

impl ProxyBuilder {
    /// Constructs a new ProxyBuilder that can be used to customize the proxy before its creation
    pub fn new(listen_addr: String, server_addr: String) -> Self {
        ProxyBuilder {
            listen_addr,
            server_addr,
            event_handlers: vec![],
        }
    }

    /// Adds an event handler to the proxy
    /// Event handlers will run in the order they were added
    pub fn with_event_handler(mut self, event_handler: Box<dyn ProxyEventHandler + Send + Sync>) -> ProxyBuilder {
        self.event_handlers.push(event_handler);
        self
    }

    /// Build the proxy
    pub fn build(self) -> Proxy {
        Proxy {
            listen_addr: self.listen_addr,
            server_addr: self.server_addr,
            event_handlers: Arc::new(self.event_handlers),
        }
    }

    /// Build the proxy and run it
    pub async fn build_and_run(self) -> anyhow::Result<()> {
        self.build().run().await
    }
}

/// Proxy using the TibiaCodec to proxy frames between clients and a server
/// Handles multiple connections
/// Can be extended using the trait ProxyEventHandler and by adding them using add_event_handler() before running the proxy
pub struct Proxy {
    listen_addr: String,
    server_addr: String,
    event_handlers: Arc<Vec<Box<dyn ProxyEventHandler + Send + Sync>>>,
}

impl Proxy {
    /// Build a proxy
    pub fn builder(listen_addr: String, server_addr: String) -> ProxyBuilder {
        ProxyBuilder::new(listen_addr, server_addr)
    }

    /// Starts the proxy and consumes self
    pub async fn run(self) -> anyhow::Result<()> { // -> ProxyResult
        let listener = TcpListener::bind(self.listen_addr).await?;
        
        let mut connection_id = 0;
        while let Ok((inbound, _)) = listener.accept().await {
            let connection = ProxyConnection {
                id: connection_id,
                client_addr: inbound.peer_addr()?.to_string(),
                server_addr: self.server_addr.clone(),
                event_handlers: Arc::clone(&self.event_handlers),
                frame_type: FrameType::Raw,
                current_frame_id: 0,
            };

            tokio::spawn(connection.run(inbound));
            connection_id += 1;
        }

        Ok(())
    }
}

/// A proxy connection
pub struct ProxyConnection {
    id: usize,
    client_addr: String,
    server_addr: String,
    event_handlers: Arc<Vec<Box<dyn ProxyEventHandler + Send + Sync>>>,
    frame_type: FrameType,
    current_frame_id: usize,
}

impl ProxyConnection {
    /// Returns the connection id
    pub fn id(&self) -> usize { self.id }

    /// Returns the client address
    pub fn client_addr(&self) -> &str { &self.client_addr }

    /// Returns the server address
    pub fn server_addr(&self) -> &str { &self.server_addr }
    
    /// Returns the address of the supplied origin
    pub fn addr(&self, origin: Origin) -> &str {
        match origin {
            Origin::Server => &self.server_addr,
            Origin::Client => &self.client_addr,
        }
    }

    /// Returns the id of the current frame (0 means first frame, 1 is second etc)
    pub fn current_frame_id(&self) -> usize {
        self.current_frame_id
    }

    /// Returns true if we are currently (or about to) processing the first frame
    pub fn first_frame(&self) -> bool {
        self.current_frame_id == 0
    }

    /// Returns the frame type used by the TibiaCodec in the proxy
    pub fn frame_type(&self) -> FrameType {
        self.frame_type
    }

    /// Changes the frame type of the TibiaCodec (for the next frame onwards)
    pub fn set_frame_type(&mut self, frame_type: FrameType) {
        self.frame_type = frame_type;
    }

    /// Runs the proxy by calling proxy()
    /// Triggers the on_disconnect handlers with the result
    async fn run(mut self, inbound: TcpStream) {
        let disconnect_reason = match self.proxy(inbound).await {
            Ok(origin) => DisconnectReason::DisconnectedBy(origin),
            Err(e) => DisconnectReason::Error(e),
        };

        for event_handler in self.event_handlers.clone().iter() {
            event_handler.on_disconnect(&mut self, &disconnect_reason);
        }
    }

    /// Starts the proxying
    /// Any errors will result in a disconnect and will be propagated to the caller
    async fn proxy(&mut self, inbound: TcpStream) -> anyhow::Result<Origin> {
        let event_handlers = self.event_handlers.clone();

        for event_handler in event_handlers.iter() {
            event_handler.on_new_connection(self)?;
        }

        let outbound = TcpStream::connect(self.server_addr.clone()).await?;
        let mut server = Framed::new(outbound, TibiaCodec::new());
        let mut client = Framed::new(inbound, TibiaCodec::new());

        for event_handler in event_handlers.iter() {
            event_handler.on_ready(self)?;
        }
        
        loop {
            if self.frame_type != server.codec().frame_type() || self.frame_type != client.codec().frame_type() {
                server.codec_mut().set_frame_type(self.frame_type);
                client.codec_mut().set_frame_type(self.frame_type);
            }

            let (frame, origin) = tokio::select! {
                frame = client.next() => (frame, Origin::Client),
                frame = server.next() => (frame, Origin::Server),
            };

            if let Some(frame) = frame {
                let mut frame = frame?;

                // Call middleware
                for event_handler in event_handlers.iter() {
                    frame = event_handler.on_frame(self, origin, frame)?;
                }

                // Send the frame to its destination
                let frame: Bytes = frame.into();
                match origin {
                    Origin::Client => server.send(frame).await?,
                    Origin::Server => client.send(frame).await?,
                };
            } else {
                // Disconnect by <origin>
                return Ok(origin);
            }

            self.current_frame_id += 1;
        }
    }
}
