use std::{
    convert::Infallible,
    path::PathBuf,
    pin::Pin,
    process::exit,
    task::{Context, Poll},
};

use aws_sdk_s3::{primitives::SdkBody, Client};
use aws_smithy_runtime_api::http::Request;
use bytes::Bytes;
use http_body::{Body, SizeHint};
use log::{debug, info};

struct ProgressTracker {
    percent_checkpoint: f64,
    bytes_written: u64,
    content_length: u64,
}

impl ProgressTracker {
    fn track(&mut self, len: u64) {
        self.bytes_written += len;
        let progress = self.bytes_written as f64 / self.content_length as f64;
        if progress - self.percent_checkpoint < 0.01 {
            return;
        }
        self.percent_checkpoint = progress.clone();
        info!("Upload progress: {:.2}%", progress * 100.0);
    }
}
// snippet-end:[s3.rust.ProgressTracker]

// snippet-start:[s3.rust.put-object-progress-body]
// A ProgressBody to wrap any http::Body with upload progress information.
#[pin_project::pin_project]
pub struct ProgressBody<InnerBody> {
    #[pin]
    inner: InnerBody,
    // prograss_tracker is a separate field so it can be accessed as &mut.
    progress_tracker: ProgressTracker,
}

// For an SdkBody specifically, the ProgressTracker swap itself in-place while customizing the SDK operation.
impl ProgressBody<SdkBody> {
    // Wrap a Requests's SdkBody with a new ProgressBody, and replace it on the fly.
    // This is specialized for SdkBody specifically, as SdkBody provides ::taken() to
    // swap out the current body for a fresh, empty body and then provides ::from_dyn()
    // to get an SdkBody back from the ProgressBody it created. http::Body does not have
    // this "change the wheels on the fly" utility.
    pub fn replace(value: Request<SdkBody>) -> Result<Request<SdkBody>, Infallible> {
        let value = value.map(|body| {
            let len = body.content_length().expect("upload body sized"); // TODO - panics
            let body = ProgressBody::new(body, len);
            SdkBody::from_body_0_4(body)
        });
        Ok(value)
    }
}

impl<InnerBody> ProgressBody<InnerBody>
where
    InnerBody: Body<Data = Bytes, Error = aws_smithy_types::body::Error>,
{
    pub fn new(body: InnerBody, content_length: u64) -> Self {
        Self {
            inner: body,
            progress_tracker: ProgressTracker {
                percent_checkpoint: 0.0,
                bytes_written: 0,
                content_length,
            },
        }
    }
}

impl<InnerBody> Body for ProgressBody<InnerBody>
where
    InnerBody: Body<Data = Bytes, Error = aws_smithy_types::body::Error>,
{
    type Data = Bytes;

    type Error = aws_smithy_types::body::Error;

    // Our poll_data delegates to the inner poll_data, but needs a project() to
    // get there. When the poll has data, it updates the progress_tracker.
    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();
        match this.inner.poll_data(cx) {
            Poll::Ready(Some(Ok(data))) => {
                this.progress_tracker.track(data.len() as u64);
                Poll::Ready(Some(Ok(data)))
            }
            Poll::Ready(None) => {
                debug!("done");
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending,
        }
    }

    // Delegate utilities to inner and progress_tracker.
    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        self.project().inner.poll_trailers(cx)
    }

    fn size_hint(&self) -> http_body::SizeHint {
        SizeHint::with_exact(self.progress_tracker.content_length)
    }
}
