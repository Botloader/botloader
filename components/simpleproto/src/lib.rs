use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::mpsc,
};

pub async fn read_message<T: DeserializeOwned>(
    src: &mut (impl AsyncRead + Unpin),
) -> std::io::Result<T> {
    let len = src.read_u32().await?;

    let mut payload_buf = vec![0; len as usize];
    src.read_exact(&mut payload_buf).await?;

    let decoded = serde_json::from_slice(&payload_buf)?;
    Ok(decoded)
}

pub async fn write_message<T: Serialize>(
    msg: &T,
    dst: &mut (impl AsyncWrite + Unpin),
) -> std::io::Result<()> {
    let encoded = serde_json::to_string(msg)?;
    assert!(encoded.len() < 0xffffffff);

    dst.write_u32(encoded.len() as u32).await?;
    dst.write_all(encoded.as_bytes()).await?;

    Ok(())
}

pub async fn message_writer<T: Serialize>(
    dst: &mut (impl AsyncWrite + Unpin),
    mut rx: mpsc::UnboundedReceiver<T>,
) -> std::io::Result<()> {
    while let Some(next) = rx.recv().await {
        write_message(&next, dst).await?
    }

    Ok(())
}

pub async fn message_reader<T: DeserializeOwned>(
    src: &mut (impl AsyncRead + Unpin),
    tx: mpsc::UnboundedSender<T>,
) -> std::io::Result<()> {
    loop {
        match read_message(src).await {
            Ok(msg) => {
                if tx.send(msg).is_err() {
                    return Ok(());
                }
            }
            Err(err) => return Err(err),
        }
    }
}
