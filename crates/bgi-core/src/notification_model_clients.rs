use super::{
    NotificationDispatchError, NotificationEmailRequest, NotificationHttpRequest,
    NotificationHttpResponse, NotificationWindowsToastRequest,
};

pub trait NotificationHttpClient {
    fn send(
        &mut self,
        request: &NotificationHttpRequest,
    ) -> std::result::Result<NotificationHttpResponse, NotificationDispatchError>;
}

pub trait NotificationWebSocketClient {
    fn send_text(
        &mut self,
        endpoint: &str,
        text: &str,
    ) -> std::result::Result<(), NotificationDispatchError>;
}

pub trait NotificationEmailClient {
    fn send_email(
        &mut self,
        request: &NotificationEmailRequest,
    ) -> std::result::Result<(), NotificationDispatchError>;
}

pub trait NotificationWindowsToastClient {
    fn show_toast(
        &mut self,
        request: &NotificationWindowsToastRequest,
    ) -> std::result::Result<(), NotificationDispatchError>;
}

#[derive(Debug, Default)]
pub struct RecordingNotificationHttpClient {
    pub requests: Vec<NotificationHttpRequest>,
    pub responses: Vec<NotificationHttpResponse>,
}

impl RecordingNotificationHttpClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_response(status: u16, body: impl Into<String>) -> Self {
        Self {
            requests: Vec::new(),
            responses: vec![NotificationHttpResponse {
                status,
                body: body.into(),
            }],
        }
    }

    pub fn with_responses(responses: Vec<NotificationHttpResponse>) -> Self {
        Self {
            requests: Vec::new(),
            responses,
        }
    }
}

impl NotificationHttpClient for RecordingNotificationHttpClient {
    fn send(
        &mut self,
        request: &NotificationHttpRequest,
    ) -> std::result::Result<NotificationHttpResponse, NotificationDispatchError> {
        self.requests.push(request.clone());
        if self.responses.is_empty() {
            return Ok(NotificationHttpResponse {
                status: 200,
                body: "{}".to_string(),
            });
        }
        Ok(self.responses.remove(0))
    }
}

#[derive(Debug, Default)]
pub struct RecordingNotificationWebSocketClient {
    pub messages: Vec<(String, String)>,
    pub fail_with: Option<NotificationDispatchError>,
}

impl RecordingNotificationWebSocketClient {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NotificationWebSocketClient for RecordingNotificationWebSocketClient {
    fn send_text(
        &mut self,
        endpoint: &str,
        text: &str,
    ) -> std::result::Result<(), NotificationDispatchError> {
        if let Some(error) = self.fail_with.clone() {
            return Err(error);
        }
        self.messages.push((endpoint.to_string(), text.to_string()));
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct UnsupportedNotificationWebSocketClient;

impl NotificationWebSocketClient for UnsupportedNotificationWebSocketClient {
    fn send_text(
        &mut self,
        _endpoint: &str,
        _text: &str,
    ) -> std::result::Result<(), NotificationDispatchError> {
        Err(NotificationDispatchError::UnsupportedProvider("WebSocket"))
    }
}

#[derive(Debug, Default)]
pub struct RecordingNotificationEmailClient {
    pub emails: Vec<NotificationEmailRequest>,
    pub fail_with: Option<NotificationDispatchError>,
}

impl RecordingNotificationEmailClient {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NotificationEmailClient for RecordingNotificationEmailClient {
    fn send_email(
        &mut self,
        request: &NotificationEmailRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        if let Some(error) = self.fail_with.clone() {
            return Err(error);
        }
        self.emails.push(request.clone());
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct UnsupportedNotificationEmailClient;

impl NotificationEmailClient for UnsupportedNotificationEmailClient {
    fn send_email(
        &mut self,
        _request: &NotificationEmailRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        Err(NotificationDispatchError::UnsupportedProvider("Email"))
    }
}

#[derive(Debug, Default)]
pub struct RecordingNotificationWindowsToastClient {
    pub toasts: Vec<NotificationWindowsToastRequest>,
    pub fail_with: Option<NotificationDispatchError>,
}

impl RecordingNotificationWindowsToastClient {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NotificationWindowsToastClient for RecordingNotificationWindowsToastClient {
    fn show_toast(
        &mut self,
        request: &NotificationWindowsToastRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        if let Some(error) = self.fail_with.clone() {
            return Err(error);
        }
        self.toasts.push(request.clone());
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct UnsupportedNotificationWindowsToastClient;

impl NotificationWindowsToastClient for UnsupportedNotificationWindowsToastClient {
    fn show_toast(
        &mut self,
        _request: &NotificationWindowsToastRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        Err(NotificationDispatchError::UnsupportedProvider(
            "Windows UWP",
        ))
    }
}
