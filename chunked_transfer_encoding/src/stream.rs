use core::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{stream::FusedStream, Stream};
use pin_project_lite::pin_project;

use crate::utils;

//
pin_project! {
    pub struct ChunkedStream<St> {
        #[pin]
        inner: St,
        is_final: bool,
    }
}

impl<St> fmt::Display for ChunkedStream<St>
where
    St: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChunkedStream")
            .field("inner", &self.inner)
            .field("is_final", &self.is_final)
            .finish()
    }
}

//
impl<St> ChunkedStream<St> {
    pub fn new(inner: St) -> Self {
        Self {
            inner,
            is_final: false,
        }
    }
}

//
impl<T, St> FusedStream for ChunkedStream<St>
where
    T: Into<Vec<u8>>,
    St: Stream<Item = T> + Send + 'static,
    St: FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

//
impl<T, St> Stream for ChunkedStream<St>
where
    T: Into<Vec<u8>>,
    St: Stream<Item = T> + Send + 'static,
{
    type Item = Vec<u8>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if *this.is_final {
            return Poll::Ready(None);
        }

        match this.inner.poll_next(cx) {
            Poll::Ready(Some(item)) => {
                let mut bytes: Vec<u8> = item.into();

                if bytes.is_empty() {
                    *this.is_final = true;
                }

                let chunk_size = utils::chunk_size(&bytes);
                let chunk_size_len = chunk_size.as_bytes().len();
                for c in chunk_size.as_bytes().iter().rev() {
                    bytes.insert(0, *c);
                }
                bytes.insert(chunk_size_len, b'\r');
                bytes.insert(chunk_size_len + 1, b'\n');

                bytes.extend_from_slice(b"\r\n");

                Poll::Ready(Some(bytes))
            }
            Poll::Ready(None) => {
                *this.is_final = true;

                let bytes = b"0\r\n\r\n".to_vec();

                Poll::Ready(Some(bytes))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures_util::{stream, StreamExt as _};

    #[test]
    fn test_chunked_stream() {
        futures_executor::block_on(async {
            //
            let mut st =
                ChunkedStream::new(stream::iter(vec!["Wiki", "pedia ", "in \r\n\r\nchunks."]));

            assert_eq!(st.next().await, Some(b"4\r\nWiki\r\n".to_vec()));
            assert_eq!(st.next().await, Some(b"6\r\npedia \r\n".to_vec()));
            assert_eq!(
                st.next().await,
                Some(b"E\r\nin \r\n\r\nchunks.\r\n".to_vec())
            );
            assert_eq!(st.next().await, Some(b"0\r\n\r\n".to_vec()));
            assert_eq!(st.next().await, None);

            //
            let mut st = ChunkedStream::new(stream::iter(vec![
                "Wiki",
                "pedia ",
                "in \r\n\r\nchunks.",
                "",
            ]));

            assert_eq!(st.next().await, Some(b"4\r\nWiki\r\n".to_vec()));
            assert_eq!(st.next().await, Some(b"6\r\npedia \r\n".to_vec()));
            assert_eq!(
                st.next().await,
                Some(b"E\r\nin \r\n\r\nchunks.\r\n".to_vec())
            );
            assert_eq!(st.next().await, Some(b"0\r\n\r\n".to_vec()));
            assert_eq!(st.next().await, None);
        })
    }
}
