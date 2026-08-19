use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const CHUNK_SIZE: usize = 16 * 1024;

pub async fn copy_file_chunked<W: AsyncWrite + Unpin + ?Sized>(
    path: &std::path::Path,
    writer: &mut W,
    mut on_chunk: impl FnMut(&[u8]),
) -> std::io::Result<u64> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut total: u64 = 0;

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        on_chunk(&buf[..n]);
        writer.write_all(&buf[..n]).await?;
        total += n as u64;
    }

    Ok(total)
}
