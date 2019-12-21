use std::ops::{Deref, DerefMut};

use cookie::Cookie;
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
    /// This consumes the flash message (calling in twice for the same request will yield `None`).
    fn get_flash_message(&mut self) -> Option<FlashMessage>;

    /// Serializes the message and sets it as a flash cookie for the next request.
    fn set_flash_message(&mut self, message: FlashMessage) -> Option<()>;
}

impl<State> FlashExt for Request<State> {
    fn get_flash_message(&mut self) -> Option<FlashMessage> {
        let cookie = self.get_cookie(COOKIE_NAME)?;
        let percent_decoded: Vec<u8> =
            percent_encoding::percent_decode_str(cookie.value()).collect();
        let payload = base64::decode(percent_decoded.as_slice()).ok()?;

        let cookie = Cookie::build(COOKIE_NAME, "none")
            .path("/")
            .http_only(true)
            .expires(time::at_utc(time::Timespec::new(0, 0)))
            .finish();
        self.set_cookie(cookie)?;
        Some(FlashMessage(payload))
    }

    fn set_flash_message(&mut self, message: FlashMessage) -> Option<()> {
        let message = base64::encode(message.as_slice());
        let cookie = Cookie::build(COOKIE_NAME, message)
            .path("/")
            .http_only(true)
            .finish();

        self.set_cookie(cookie)?;
        Some(())
    }
}
