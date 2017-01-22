use std::sync::{Arc, Mutex};

use opcua_core::types::*;
use opcua_core::comms::*;

use services::discovery::*;
use services::session::*;
use services::view::*;
use server::ServerState;
use tcp_session::SessionState;

/// Processes and dispatches messages for handling
pub struct MessageHandler {
    /// Server state
    server_state: ServerState,
    /// Session state
    session_state: Arc<Mutex<SessionState>>,
    /// Discovery service
    discovery_service: DiscoveryService,
    /// Session service
    session_service: SessionService,
    /// View service
    view_service: ViewService,
}

impl MessageHandler {
    pub fn new(server_state: &ServerState, session_state: &Arc<Mutex<SessionState>>) -> MessageHandler {
        MessageHandler {
            server_state: server_state.clone(),
            session_state: session_state.clone(),
            discovery_service: DiscoveryService::new(),
            session_service: SessionService::new(),
            view_service: ViewService::new(),
        }
    }

    pub fn handle_message(&mut self, message: &SupportedMessage) -> Result<SupportedMessage, &'static StatusCode> {
        let mut server_state = &mut self.server_state;
        let mut session_state = self.session_state.lock().unwrap();
        let mut session_state = &mut session_state;

        let response = match *message {
            SupportedMessage::GetEndpointsRequest(ref request) => {
                self.discovery_service.get_endpoints(server_state, session_state, request)?
            },
            SupportedMessage::CreateSessionRequest(ref request) => {
                self.session_service.create_session(server_state, session_state, request)?
            },
            SupportedMessage::CloseSessionRequest(ref request) => {
                self.session_service.close_session(server_state, session_state, request)?
            },
            SupportedMessage::ActivateSessionRequest(ref request) => {
                self.session_service.activate_session(server_state, session_state, request)?
            },
            SupportedMessage::BrowseRequest(ref request) => {
                self.view_service.browse(server_state, session_state, request)?
            },
            _ => {
                debug!("Message handler does not handle this kind of message");
                return Err(&BAD_SERVICE_UNSUPPORTED);
            }
        };
        Ok(response)
    }
}
