use crate::TRANSFER_ENCODING_CHUNKED_VALUE;

pub fn transfer_encoding_value_is_chunked(value: &str) -> bool {
    value == TRANSFER_ENCODING_CHUNKED_VALUE
        || value
            .split(',')
            .map(|s| s.trim())
            .any(|x| x == TRANSFER_ENCODING_CHUNKED_VALUE)
}

#[cfg(feature = "http")]
pub fn transfer_encoding_is_chunked(headers: &http::HeaderMap<http::HeaderValue>) -> bool {
    if let Some(header_value) = headers.get(http::header::TRANSFER_ENCODING) {
        if let Ok(value) = core::str::from_utf8(header_value.as_bytes()) {
            return transfer_encoding_value_is_chunked(value);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_encoding_value_is_chunked() {
        assert!(transfer_encoding_value_is_chunked("chunked"));
        assert!(transfer_encoding_value_is_chunked("gzip, chunked"));
        assert!(!transfer_encoding_value_is_chunked("compress"));
    }

    #[cfg(feature = "http")]
    #[test]
    fn test_transfer_encoding_is_chunked() {
        assert!(transfer_encoding_is_chunked(
            http::Response::builder()
                .header("Transfer-Encoding", "chunked")
                .body(())
                .unwrap()
                .headers()
        ));
        assert!(!transfer_encoding_is_chunked(
            http::Response::builder()
                .header("Transfer-Encoding", "compress")
                .body(())
                .unwrap()
                .headers()
        ));
        assert!(!transfer_encoding_is_chunked(
            http::Response::builder().body(()).unwrap().headers()
        ));
    }
}
