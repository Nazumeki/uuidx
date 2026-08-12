use js_sys::{Error, Reflect};
use wasm_bindgen::JsValue;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ApiError {
    code: &'static str,
    message: String,
}

impl ApiError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub(crate) fn into_js(self) -> JsValue {
        let error = Error::new(&self.message);
        let _ = Reflect::set(
            error.as_ref(),
            &JsValue::from_str("code"),
            &JsValue::from_str(self.code),
        );
        error.into()
    }

    #[cfg(test)]
    pub(crate) fn code(&self) -> &'static str {
        self.code
    }
}

impl From<uuidx_core::ParseUuidError> for ApiError {
    fn from(error: uuidx_core::ParseUuidError) -> Self {
        Self::new("invalid_uuid", error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_retain_machine_readable_codes() {
        let error = ApiError::new("invalid_options", "bad options");
        assert_eq!(error.code(), "invalid_options");
        assert_eq!(error.message, "bad options");
    }

    #[test]
    fn uuid_parse_errors_have_the_public_error_code() {
        let parse_error = uuidx_core::parse_uuid("not-a-uuid").unwrap_err();
        let error = ApiError::from(parse_error);

        assert_eq!(error.code(), "invalid_uuid");
        assert!(!error.message.is_empty());
    }
}
