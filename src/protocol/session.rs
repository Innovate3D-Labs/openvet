//! One WebSocket, one JSON text frame per envelope, subprotocol `openvet.v0`.
//!
//! Ping frames are answered. Reconnect is the host's job. Message ids are
//! remembered for the life of this connection, up to a fixed cap, and are
//! forgotten when the socket closes. The 24-hour replay window in the spec is
//! a deployment concern across restarts. This type does not persist it.

use crate::envelope::Envelope;
use crate::error::{ErrorCode, ProtocolError};
use crate::security::{enforce_direction, SessionConfig};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::fmt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::{HeaderValue, Response as HttpResponse, StatusCode};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_hdr_async, connect_async, MaybeTlsStream, WebSocketStream};

const SUBPROTOCOL: &str = "openvet.v0";
const MAX_SEEN_IDS: usize = 4096;

/// A live session. `S` is the byte stream under the WebSocket, usually TCP or a TLS stream.
pub struct Session<S> {
    ws: WebSocketStream<S>,
    config: SessionConfig,
    seen: HashSet<String>,
}

impl Session<MaybeTlsStream<TcpStream>> {
    /// Open a plain `ws://` connection. Production traffic should be `wss`,
    /// built by the host and passed to [`Session::from_ws`].
    pub async fn connect(url: &str, config: SessionConfig) -> Result<Self, SessionError> {
        let mut request = url.into_client_request().map_err(SessionError::transport)?;
        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_static(SUBPROTOCOL),
        );
        let (ws, _) = connect_async(request)
            .await
            .map_err(SessionError::transport)?;
        Ok(Self::from_ws(ws, config))
    }
}

impl<S> Session<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn from_ws(ws: WebSocketStream<S>, config: SessionConfig) -> Self {
        Self {
            ws,
            config,
            seen: HashSet::new(),
        }
    }

    /// WebSocket handshake on a stream the host already accepted.
    ///
    /// If that stream is a TLS session, the host did the TLS. This function
    /// only checks that the client asked for `openvet.v0`.
    pub async fn accept(stream: S, config: SessionConfig) -> Result<Self, SessionError> {
        let ws = accept_hdr_async(stream, select_protocol)
            .await
            .map_err(SessionError::transport)?;
        Ok(Self::from_ws(ws, config))
    }

    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    pub async fn send(&mut self, envelope: &Envelope) -> Result<(), SessionError> {
        enforce_direction(envelope, &self.config.local, &self.config.remote, false)?;
        self.config.caps.check(&envelope.msg)?;
        let bytes = envelope.to_vec()?;
        let text = String::from_utf8(bytes)
            .map_err(|_| ProtocolError::new(ErrorCode::Malformed, "envelope is not valid utf-8"))?;
        self.ws
            .send(Message::Text(text.into()))
            .await
            .map_err(SessionError::transport)?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Envelope, SessionError> {
        loop {
            let frame = match self.ws.next().await {
                Some(frame) => frame.map_err(SessionError::transport)?,
                None => return Err(SessionError::Closed),
            };
            match frame {
                Message::Text(text) => {
                    let envelope = Envelope::from_slice(text.as_bytes())?;
                    self.remember_inbound(&envelope)?;
                    return Ok(envelope);
                }
                Message::Ping(payload) => {
                    self.ws
                        .send(Message::Pong(payload))
                        .await
                        .map_err(SessionError::transport)?;
                }
                Message::Pong(_) => {}
                Message::Close(_) => return Err(SessionError::Closed),
                Message::Binary(_) | Message::Frame(_) => return Err(SessionError::NonText),
            }
        }
    }

    fn remember_inbound(&mut self, envelope: &Envelope) -> Result<(), SessionError> {
        enforce_direction(envelope, &self.config.local, &self.config.remote, true)?;
        self.config.caps.check(&envelope.msg)?;
        if self.seen.contains(&envelope.id) {
            return Err(ProtocolError::new(ErrorCode::Conflict, "repeated message id").into());
        }
        if self.seen.len() >= MAX_SEEN_IDS {
            return Err(ProtocolError::new(
                ErrorCode::Limit,
                "session replay window is full; reconnect",
            )
            .into());
        }
        self.seen.insert(envelope.id.clone());
        Ok(())
    }
}

/// Convenience for examples and tests. Same as [`Session::connect`].
pub async fn connect(
    url: &str,
    config: SessionConfig,
) -> Result<Session<MaybeTlsStream<TcpStream>>, SessionError> {
    Session::connect(url, config).await
}

/// Convenience for examples and tests. Same as [`Session::accept`].
pub async fn accept<S>(stream: S, config: SessionConfig) -> Result<Session<S>, SessionError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    Session::accept(stream, config).await
}

// The handshake callback's error type is a full HTTP response. That is the
// tungstenite API; boxing it would no longer match the trait.
#[allow(clippy::result_large_err)]
fn select_protocol(request: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
    let offered = request
        .headers()
        .get("Sec-WebSocket-Protocol")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|header| header.split(',').any(|part| part.trim() == SUBPROTOCOL));
    if !offered {
        let rejection = HttpResponse::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Some(format!("{SUBPROTOCOL} required")))
            .expect("rejection response is static");
        return Err(rejection);
    }
    response.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static(SUBPROTOCOL),
    );
    Ok(response)
}

/// A session failure. Protocol rejects stay [`ProtocolError`] so a host can branch on [`ErrorCode`].
#[derive(Debug)]
pub enum SessionError {
    Protocol(ProtocolError),
    Transport(String),
    Closed,
    NonText,
}

impl SessionError {
    fn transport(err: impl fmt::Display) -> Self {
        let mut detail = err.to_string();
        const MAX: usize = 180;
        if detail.len() > MAX {
            detail.truncate(MAX);
        }
        Self::Transport(detail)
    }
}

impl From<ProtocolError> for SessionError {
    fn from(err: ProtocolError) -> Self {
        Self::Protocol(err)
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(err) => write!(f, "{err}"),
            Self::Transport(detail) => write!(f, "transport: {detail}"),
            Self::Closed => f.write_str("connection closed"),
            Self::NonText => f.write_str("peer sent a non-text websocket frame"),
        }
    }
}

impl std::error::Error for SessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Protocol(err) => Some(err),
            _ => None,
        }
    }
}
