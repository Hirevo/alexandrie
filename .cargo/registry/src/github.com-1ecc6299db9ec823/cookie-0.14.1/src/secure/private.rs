extern crate aes_gcm;

use self::aes_gcm::Aes256Gcm;
use self::aes_gcm::aead::{Aead, NewAead, generic_array::GenericArray, Payload};

use crate::secure::{base64, rand, Key};
use crate::{Cookie, CookieJar};

use self::rand::RngCore;

// Keep these in sync, and keep the key len synced with the `private` docs as
// well as the `KEYS_INFO` const in secure::Key.
pub(crate) const NONCE_LEN: usize = 12;
pub(crate) const TAG_LEN: usize = 16;
pub(crate) const KEY_LEN: usize = 32;

/// A child cookie jar that provides authenticated encryption for its cookies.
///
/// A _private_ child jar signs and encrypts all the cookies added to it and
/// verifies and decrypts cookies retrieved from it. Any cookies stored in a
/// `PrivateJar` are simultaneously assured confidentiality, integrity, and
/// authenticity. In other words, clients cannot discover nor tamper with the
/// contents of a cookie, nor can they fabricate cookie data.
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "private")))]
pub struct PrivateJar<'a> {
    parent: &'a mut CookieJar,
    key: [u8; KEY_LEN]
}

impl<'a> PrivateJar<'a> {
    /// Creates a new child `PrivateJar` with parent `parent` and key `key`.
    /// This method is typically called indirectly via the `signed` method of
    /// `CookieJar`.
    pub(crate) fn new(parent: &'a mut CookieJar, key: &Key) -> PrivateJar<'a> {
        PrivateJar { parent, key: key.encryption }
    }

    /// Given a sealed value `str` and a key name `name`, where the nonce is
    /// prepended to the original value and then both are Base64 encoded,
    /// verifies and decrypts the sealed value and returns it. If there's a
    /// problem, returns an `Err` with a string describing the issue.
    fn unseal(&self, name: &str, value: &str) -> Result<String, &'static str> {
        let data = base64::decode(value).map_err(|_| "bad base64 value")?;
        if data.len() <= NONCE_LEN {
            return Err("length of decoded data is <= NONCE_LEN");
        }

        let (nonce, cipher) = data.split_at(NONCE_LEN);
        let payload = Payload { msg: cipher, aad: name.as_bytes() };

        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&self.key));
        aead.decrypt(GenericArray::from_slice(nonce), payload)
            .map_err(|_| "invalid key/nonce/value: bad seal")
            .and_then(|s| String::from_utf8(s).map_err(|_| "bad unsealed utf8"))
    }

    /// Returns a reference to the `Cookie` inside this jar with the name `name`
    /// and authenticates and decrypts the cookie's value, returning a `Cookie`
    /// with the decrypted value. If the cookie cannot be found, or the cookie
    /// fails to authenticate or decrypt, `None` is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cookie::{CookieJar, Cookie, Key};
    ///
    /// let key = Key::generate();
    /// let mut jar = CookieJar::new();
    /// let mut private_jar = jar.private(&key);
    /// assert!(private_jar.get("name").is_none());
    ///
    /// private_jar.add(Cookie::new("name", "value"));
    /// assert_eq!(private_jar.get("name").unwrap().value(), "value");
    /// ```
    pub fn get(&self, name: &str) -> Option<Cookie<'static>> {
        if let Some(cookie_ref) = self.parent.get(name) {
            let mut cookie = cookie_ref.clone();
            if let Ok(value) = self.unseal(name, cookie.value()) {
                cookie.set_value(value);
                return Some(cookie);
            }
        }

        None
    }

    /// Adds `cookie` to the parent jar. The cookie's value is encrypted with
    /// authenticated encryption assuring confidentiality, integrity, and
    /// authenticity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cookie::{CookieJar, Cookie, Key};
    ///
    /// let key = Key::generate();
    /// let mut jar = CookieJar::new();
    /// jar.private(&key).add(Cookie::new("name", "value"));
    ///
    /// assert_ne!(jar.get("name").unwrap().value(), "value");
    /// assert_eq!(jar.private(&key).get("name").unwrap().value(), "value");
    /// ```
    pub fn add(&mut self, mut cookie: Cookie<'static>) {
        self.encrypt_cookie(&mut cookie);
        self.parent.add(cookie);
    }

    /// Adds an "original" `cookie` to parent jar. The cookie's value is
    /// encrypted with authenticated encryption assuring confidentiality,
    /// integrity, and authenticity. Adding an original cookie does not affect
    /// the [`CookieJar::delta()`] computation. This method is intended to be
    /// used to seed the cookie jar with cookies received from a client's HTTP
    /// message.
    ///
    /// For accurate `delta` computations, this method should not be called
    /// after calling `remove`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cookie::{CookieJar, Cookie, Key};
    ///
    /// let key = Key::generate();
    /// let mut jar = CookieJar::new();
    /// jar.private(&key).add_original(Cookie::new("name", "value"));
    ///
    /// assert_eq!(jar.iter().count(), 1);
    /// assert_eq!(jar.delta().count(), 0);
    /// ```
    pub fn add_original(&mut self, mut cookie: Cookie<'static>) {
        self.encrypt_cookie(&mut cookie);
        self.parent.add_original(cookie);
    }

    /// Encrypts the cookie's value with authenticated encryption providing
    /// confidentiality, integrity, and authenticity.
    fn encrypt_cookie(&self, cookie: &mut Cookie) {
        // Create a vec to hold the [nonce | cookie value | tag].
        let cookie_val = cookie.value().as_bytes();
        let mut data = vec![0; NONCE_LEN + cookie_val.len() + TAG_LEN];

        // Split data into three: nonce, input/output, tag. Copy input.
        let (nonce, in_out) = data.split_at_mut(NONCE_LEN);
        let (in_out, tag) = in_out.split_at_mut(cookie_val.len());
        in_out.copy_from_slice(cookie_val);

        // Fill nonce piece with random data.
        let mut rng = self::rand::thread_rng();
        rng.try_fill_bytes(nonce).expect("couldn't random fill nonce");
        let nonce = GenericArray::clone_from_slice(nonce);

        // Perform the actual sealing operation, using the cookie's name as
        // associated data to prevent value swapping.
        let aad = cookie.name().as_bytes();
        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&self.key));
        let aad_tag = aead.encrypt_in_place_detached(&nonce, aad, in_out)
            .expect("encryption failure!");

        // Copy the tag into the tag piece.
        tag.copy_from_slice(&aad_tag);

        // Base64 encode [nonce | encrypted value | tag].
        cookie.set_value(base64::encode(&data));
    }

    /// Removes `cookie` from the parent jar.
    ///
    /// For correct removal, the passed in `cookie` must contain the same `path`
    /// and `domain` as the cookie that was initially set.
    ///
    /// See [`CookieJar::remove()] for more details.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cookie::{CookieJar, Cookie, Key};
    ///
    /// let key = Key::generate();
    /// let mut jar = CookieJar::new();
    /// let mut private_jar = jar.private(&key);
    ///
    /// private_jar.add(Cookie::new("name", "value"));
    /// assert!(private_jar.get("name").is_some());
    ///
    /// private_jar.remove(Cookie::named("name"));
    /// assert!(private_jar.get("name").is_none());
    /// ```
    pub fn remove(&mut self, cookie: Cookie<'static>) {
        self.parent.remove(cookie);
    }
}

#[cfg(test)]
mod test {
    use crate::{CookieJar, Cookie, Key};

    #[test]
    fn simple() {
        let key = Key::generate();
        let mut jar = CookieJar::new();
        assert_simple_behaviour!(jar, jar.private(&key));
    }

    #[test]
    fn secure() {
        let key = Key::generate();
        let mut jar = CookieJar::new();
        assert_secure_behaviour!(jar, jar.private(&key));
    }

    #[test]
    fn roundtrip() {
        // Secret is SHA-256 hash of 'Super secret!' passed through HKDF-SHA256.
        let key = Key::from(&[89, 202, 200, 125, 230, 90, 197, 245, 166, 249,
            34, 169, 135, 31, 20, 197, 94, 154, 254, 79, 60, 26, 8, 143, 254,
            24, 116, 138, 92, 225, 159, 60, 157, 41, 135, 129, 31, 226, 196, 16,
            198, 168, 134, 4, 42, 1, 196, 24, 57, 103, 241, 147, 201, 185, 233,
            10, 180, 170, 187, 89, 252, 137, 110, 107]);

        let mut jar = CookieJar::new();
        jar.add(Cookie::new("encrypted_with_ring014",
                "lObeZJorGVyeSWUA8khTO/8UCzFVBY9g0MGU6/J3NN1R5x11dn2JIA=="));
        jar.add(Cookie::new("encrypted_with_ring016",
                "SU1ujceILyMBg3fReqRmA9HUtAIoSPZceOM/CUpObROHEujXIjonkA=="));

        let private = jar.private(&key);
        assert_eq!(private.get("encrypted_with_ring014").unwrap().value(), "Tamper-proof");
        assert_eq!(private.get("encrypted_with_ring016").unwrap().value(), "Tamper-proof");
    }
}
