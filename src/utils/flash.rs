use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tide::Request;

use crate::utils::cookies::CookiesExt;

/// Flash cookie's name.
pub const COOKIE_NAME: &str = "flash";

/// Represents a flash message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlashMessage(Vec<u8>);

impl FlashMessage {
    /// Construct a flash message from a JSON-serializable type.
    pub fn from_json<T>(msg: &T) -> Result<FlashMessage, json::Error>
    where
        T: Serialize,
    {
        let data = json::to_vec(msg)?;
        Ok(FlashMessage(data))
    }

    /// Deserialize a flash message (from JSON).
    pub fn parse_json<'a, T>(&'a self) -> Result<T, json::Error>
    where
        T: Deserialize<'a>,
    {
        json::from_slice(self.0.as_slice())
    }
}

impl From<Vec<u8>> for FlashMessage {
    fn from(data: Vec<u8>) -> FlashMessage {
        FlashMessage(data)
    }
}

impl AsRef<[u8]> for FlashMessage {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Deref for FlashMessage {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FlashMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A trait to extend `Context` with helper methods for manipulating flash cookies.
pub trait FlashExt {
    /// Get the received flash message for this request, if any.
    /// This consumes the flash message (calling in twice for the same request will yield `None` for the second call).
    fn get_flash_message(&mut self) -> Option<FlashMessage>;

    /// Serializes the message and sets it as a flash cookie for the next request.
    fn set_flash_message(&mut self, message: FlashMessage) -> Option<()>;
}

impl<State> FlashExt for Request<State> {
    fn get_flash_message(&mut self) -> Option<FlashMessage> {
        let data = self.ext::<FlashData>()?;
        let mut locked = data.content.lock().unwrap();
        locked.take()
    }

    fn set_flash_message(&mut self, message: FlashMessage) -> Option<()> {
        let data = self.ext::<FlashData>()?;
        let mut locked = data.content.lock().unwrap();
        locked.replace(message);
        Some(())
    }
}

/// A representation of flash cookies which wraps a `FlashMessage`.
#[derive(Debug, Clone)]
pub struct FlashData {
    /// The `FlashMessage` for the current request.
    pub content: Arc<Mutex<Option<FlashMessage>>>,
}

impl FlashData {
    /// Construct the flash data from request headers.
    pub fn from_request<State>(req: &Request<State>) -> Self {
        let flash_message = req.get_cookie(COOKIE_NAME).and_then(|cookie| {
            let percent_decoded: Vec<u8> =
                percent_encoding::percent_decode_str(cookie.value()).collect();
            let payload = base64::decode(percent_decoded.as_slice()).ok()?;
            Some(FlashMessage(payload))
        });

        FlashData {
            content: Arc::new(Mutex::new(flash_message)),
        }
    }
}
