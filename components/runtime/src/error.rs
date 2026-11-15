use std::borrow::Cow;

use deno_error::{AdditionalProperties, JsErrorBox, JsErrorClass};

/// Error type returned from ops.
///
/// deno_core requires op errors to implement [`JsErrorClass`], which `anyhow::Error` can't, so
/// this wraps an `anyhow::Error` and implements it instead. Ops can keep using `?` on
/// `anyhow::Result`s (and the other error types listed below) as before.
///
/// To throw a specific JS error class, create a [`JsErrorBox`] (see [`custom_error`]) and turn it
/// into an `anyhow::Error`, the class is recovered from it here.
#[derive(Debug)]
pub struct OpError(pub anyhow::Error);

impl OpError {
    pub fn into_inner(self) -> anyhow::Error {
        self.0
    }
}

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // alternate format includes the context chain, matching the message deno_core used to
        // produce for anyhow errors
        write!(f, "{:#}", self.0)
    }
}

impl std::error::Error for OpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl JsErrorClass for OpError {
    fn get_class(&self) -> Cow<'static, str> {
        error_class(&self.0).unwrap_or(Cow::Borrowed(deno_error::builtin_classes::GENERIC_ERROR))
    }

    fn get_message(&self) -> Cow<'static, str> {
        match self.0.downcast_ref::<JsErrorBox>() {
            Some(js_err) => js_err.get_message(),
            None => Cow::Owned(self.to_string()),
        }
    }

    fn get_additional_properties(&self) -> AdditionalProperties {
        match self.0.downcast_ref::<JsErrorBox>() {
            Some(js_err) => js_err.get_additional_properties(),
            None => Box::new(std::iter::empty()),
        }
    }

    fn get_ref(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self.0.as_ref()
    }
}

/// Returns the JS error class of the error, if it was created with a specific one
/// (see [`custom_error`]).
pub fn error_class(err: &anyhow::Error) -> Option<Cow<'static, str>> {
    err.downcast_ref::<JsErrorBox>().map(|e| e.get_class())
}

/// Creates an error that will be thrown as the JS error class `class`.
///
/// The class needs to be registered on the JS side with `Deno.core.registerErrorClass`.
pub fn custom_error(
    class: impl Into<Cow<'static, str>>,
    message: impl Into<Cow<'static, str>>,
) -> anyhow::Error {
    JsErrorBox::new(class, message).into()
}

// A blanket `impl<E: Into<anyhow::Error>> From<E> for OpError` would conflict with
// `impl<T> From<T> for T` (OpError itself is an `std::error::Error`), so the error types used with
// `?` in ops are listed explicitly instead.
macro_rules! impl_from_for_op_error {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for OpError {
                fn from(err: $ty) -> Self {
                    Self(err.into())
                }
            }
        )*
    };
}

impl_from_for_op_error!(
    anyhow::Error,
    JsErrorBox,
    deno_core::error::ResourceError,
    std::io::Error,
    std::num::ParseIntError,
    serde_json::Error,
    url::ParseError,
    reqwest::Error,
    image::ImageError,
    twilight_http::Error,
    twilight_http::response::DeserializeBodyError,
    dbrokerapi::state_client::Error,
    stores::bucketstore::StoreError,
    stores::timers::TimerStoreError,
);

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use deno_error::JsErrorClass;

    use super::{custom_error, error_class, OpError};

    #[test]
    fn custom_error_class_is_preserved() {
        let err = custom_error("DiscordNotFoundError", "channel not found");
        assert_eq!(error_class(&err).as_deref(), Some("DiscordNotFoundError"));

        let op_err = OpError::from(err);
        assert_eq!(op_err.get_class(), "DiscordNotFoundError");
        assert_eq!(op_err.get_message(), "channel not found");
    }

    #[test]
    fn other_errors_are_generic_with_context() {
        let err: anyhow::Error = Err::<(), _>(anyhow::anyhow!("inner"))
            .context("outer")
            .unwrap_err();
        assert_eq!(error_class(&err), None);

        let op_err = OpError::from(err);
        assert_eq!(op_err.get_class(), "Error");
        assert_eq!(op_err.get_message(), "outer: inner");
    }
}
