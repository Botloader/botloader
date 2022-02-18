use std::{
    borrow::Cow, cell::RefCell, collections::HashMap, pin::Pin, rc::Rc, str::FromStr,
    time::Duration,
};

use deno_core::{
    op_async, op_sync, AsyncRefCell, AsyncResult, CancelFuture, CancelHandle, CancelTryFuture,
    Extension, OpState, RcRef, Resource, ResourceId, ZeroCopyBuf,
};
use futures::Stream;
use reqwest::Body;
use runtime_models::internal::httpclient::{ClientHttpRequest, ClientHttpResponse};
use tokio::{io::AsyncReadExt, sync::mpsc};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tokio_util::io::StreamReader;
use tracing::info;
use url::Url;
use vm::AnyError;

pub fn extension() -> Extension {
    Extension::builder()
        .ops(vec![(
            "op_bl_http_client_stream",
            op_sync(op_bl_http_client_stream),
        )])
        .ops(vec![(
            "op_bl_http_request_send",
            op_async(op_bl_http_request_send),
        )])
        .build()
}

pub fn op_bl_http_client_stream(state: &mut OpState, _: (), _: ()) -> Result<ResourceId, AnyError> {
    let (tx, rx) = mpsc::channel(2);
    let resource = RequestBodyResource {
        cancel: CancelHandle::new(),
        tx,
        rx: RefCell::new(Some(rx)),
    };

    crate::try_insert_resource_table(&mut state.resource_table, resource)
}

pub async fn op_bl_http_request_send(
    state_rc: Rc<RefCell<OpState>>,
    args: ClientHttpRequest,
    _: (),
) -> Result<ClientHttpResponse, AnyError> {
    crate::RateLimiters::ops_until_ready(state_rc.clone(), crate::RatelimiterType::UserHttp).await;

    // lookup the body stream resource
    let req_resource = if let Some(rid) = args.body_resource_id {
        let state = state_rc.borrow();
        Some(state.resource_table.get::<RequestBodyResource>(rid)?)
    } else {
        None
    };

    let parsed_url = Url::parse(&args.path)?;

    let client = { state_rc.borrow_mut().borrow::<reqwest::Client>().clone() };
    let mut builder = client.request(reqwest::Method::from_str(&args.method)?, parsed_url);

    // add headers
    for (k, v) in args.headers {
        builder = builder.header(k, v);
    }

    // set the body
    if let Some(req_resource) = req_resource {
        let rx = req_resource
            .rx
            .borrow_mut()
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed retrieving body resource stream"))?;

        builder = builder.body(Body::wrap_stream(ReceiverStream::new(rx)))
    }

    let res = builder.send().await;

    // close the req body stream
    if let Some(rid) = args.body_resource_id {
        state_rc.borrow_mut().resource_table.close(rid).ok();
    }

    handle_response(state_rc, res?)
}

fn handle_response(
    state_rc: Rc<RefCell<OpState>>,
    resp: reqwest::Response,
) -> Result<ClientHttpResponse, AnyError> {
    let mut resp_headers = HashMap::<String, String>::new();
    for (k, v) in resp.headers() {
        resp_headers.insert(k.to_string(), v.to_str()?.to_string());
    }
    let status_code = resp.status();

    // response body resource
    let stream: BytesStream = Box::pin(
        resp.bytes_stream()
            .map(|r| r.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))),
    );
    let stream_reader = StreamReader::new(stream);
    let rid = state_rc
        .borrow_mut()
        .resource_table
        .add(RequestReponseBodyResource {
            body: AsyncRefCell::new(stream_reader),
            cancel: CancelHandle::default(),
        });

    tokio::task::spawn_local(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let mut borrowed = state_rc.borrow_mut();
        if borrowed.resource_table.has(rid) {
            info!(%rid, "closing resource");
            borrowed.resource_table.close(rid).ok();
        }
    });

    Ok(ClientHttpResponse {
        body_resource_id: rid,
        headers: resp_headers,
        status_code: status_code.as_u16() as i32,
    })
}

struct RequestBodyResource {
    tx: mpsc::Sender<std::io::Result<Vec<u8>>>,
    rx: RefCell<Option<mpsc::Receiver<std::io::Result<Vec<u8>>>>>,
    cancel: CancelHandle,
}

impl Resource for RequestBodyResource {
    fn name(&self) -> Cow<str> {
        "requestBodyResource".into()
    }

    fn write(self: Rc<Self>, buf: ZeroCopyBuf) -> AsyncResult<usize> {
        Box::pin(async move {
            let data = buf.to_vec();
            let len = data.len();

            let body = RcRef::map(&self, |r| &r.tx);
            let cancel = RcRef::map(self, |r| &r.cancel);
            body.send(Ok(data))
                .or_cancel(cancel)
                .await?
                .map_err(|_| anyhow::anyhow!("body is closed"))?;

            Ok(len)
        })
    }

    fn close(self: Rc<Self>) {
        self.cancel.cancel()
    }
}

type BytesStream = Pin<Box<dyn Stream<Item = Result<bytes::Bytes, std::io::Error>> + Unpin>>;

struct RequestReponseBodyResource {
    body: AsyncRefCell<StreamReader<BytesStream, bytes::Bytes>>,
    cancel: CancelHandle,
}

impl Resource for RequestReponseBodyResource {
    fn name(&self) -> Cow<str> {
        "requestReponseBodyResource".into()
    }

    fn read(self: Rc<Self>, mut buf: ZeroCopyBuf) -> AsyncResult<usize> {
        Box::pin(async move {
            let mut reader = RcRef::map(&self, |r| &r.body).borrow_mut().await;
            let cancel = RcRef::map(self, |r| &r.cancel);
            let read = reader.read(&mut buf).try_or_cancel(cancel).await?;
            Ok(read)
        })
    }

    fn close(self: Rc<Self>) {
        self.cancel.cancel()
    }
}
