use bytes::Bytes;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct SseClientStream<T> {
    buffer: SseBuffer,
    inner: Pin<Box<dyn futures::Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    // _phantom: std::marker::PhantomData<T>,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> SseClientStream<T> {
    pub fn new<S>(stream: S) -> Self
    where
        S: futures::Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
    {
        Self {
            buffer: SseBuffer {
                lines: Vec::new(),
                current_line: Vec::new(),
            },
            inner: Box::pin(stream),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: DeserializeOwned> futures::Stream for SseClientStream<T> {
    type Item = Result<T, SseError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(event) = self.buffer.try_find_next_event() {
                return Poll::Ready(Some(event));
            }

            match self.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    self.buffer.append_bytes(bytes.into_iter());
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(e.into())));
                }
                Poll::Ready(None) => {
                    // end of stream
                    if !self.buffer.current_line.is_empty() {
                        let line = std::mem::take(&mut self.buffer.current_line);
                        self.buffer.lines.push(line);
                    }
                    if let Some(event) = self.buffer.try_find_next_event() {
                        return Poll::Ready(Some(event));
                    } else {
                        return Poll::Ready(None);
                    }
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}

pub enum SseError {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
}

impl From<reqwest::Error> for SseError {
    fn from(e: reqwest::Error) -> Self {
        SseError::Reqwest(e)
    }
}

impl From<serde_json::Error> for SseError {
    fn from(e: serde_json::Error) -> Self {
        SseError::Serde(e)
    }
}

struct SseBuffer {
    lines: Vec<Vec<u8>>,
    current_line: Vec<u8>,
}

impl SseBuffer {
    fn try_find_next_event<T: DeserializeOwned>(&mut self) -> Option<Result<T, SseError>> {
        if self.lines.is_empty() {
            return None;
        }

        let Some(empty_index) = self.lines.iter().position(|line| line.is_empty()) else {
            return None;
        };

        let event_lines = self.lines.drain(..=empty_index);
        for line in event_lines {
            if line.starts_with(b"data:") {
                let data = &line[5..];
                let data_str = String::from_utf8_lossy(data).trim().to_string();
                match serde_json::from_str(&data_str) {
                    Ok(v) => return Some(Ok(v)),
                    Err(err) => return Some(Err(SseError::Serde(err))),
                }
            }
        }

        None
    }

    fn append_bytes(&mut self, b: impl Iterator<Item = u8>) {
        for data in b {
            if data == b'\r' {
                // ignore
                continue;
            }

            if data == b'\n' {
                self.lines.push(std::mem::take(&mut self.current_line));
            } else {
                // accumulate line
                self.current_line.push(data);
            }
        }
    }
}
