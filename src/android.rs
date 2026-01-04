use std::time::Duration;

use async_compression::tokio::write::{GzipDecoder, GzipEncoder};
use futures_lite::StreamExt;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadBuf},
    net::{TcpStream, tcp::OwnedWriteHalf},
    time::sleep,
};

pub struct AndroidServer {
    pub writer: OwnedWriteHalf,
}

use tokio_util::{bytes::Buf, codec::{Decoder, FramedRead}};

struct MessageCodec;

struct Message {
    key: String,
    gzipped: Vec<u8>,
}

impl Decoder for MessageCodec {
    type Item = (String, Vec<u8>);
    type Error = anyhow::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let delimiter = b"-_-EOF-_-";
        if let Some(pos) = src
            .windows(delimiter.len())
            .position(|window| window == delimiter)
        {
            let mut data = src.split_to(pos);
            let buffer = data
                .split_to(data.iter().position(|x| x == &b'\n').unwrap())
                .to_vec();
            let key = String::from_utf8_lossy(&buffer);
            src.advance(delimiter.len());
            return Ok(Some((key.to_string(), data.to_vec())));
        }
        // Wait for more data
        Ok(None)
    }
}

impl AndroidServer {
    pub async fn new(addr: String) -> anyhow::Result<Self> {
        let (read, writer) = TcpStream::connect(addr).await?.into_split();

        tokio::spawn(async move {
            // let bf = BufReader::new(read);
            let mut buffer = Vec::new();
            let mut framed_reader = FramedRead::new(read, MessageCodec);

            while let Some(Ok((key, mut value))) = framed_reader.next().await {
                value.remove(0);
                buffer.clear();
                let mut decoder = GzipDecoder::new(&mut buffer);
                decoder.write(&value).await.unwrap();
                decoder.flush().await.unwrap();

                if key == "error" {
                    eprintln!("{}", String::from_utf8_lossy(&buffer));
                }

            }
            
        });

        Ok(Self { writer })
    }
    pub async fn dispatch(&mut self, key: &str, contents: Vec<u8>) -> anyhow::Result<()> {
        self.writer.write(key.as_bytes()).await?;
        self.writer.write(b"\n").await?;

        let mut buffer = vec![];
        let mut encoder = GzipEncoder::new(&mut buffer);
        encoder.write(&contents).await?;
        encoder.shutdown().await?;
        self.writer.write(&buffer).await?;
        self.writer.write(b"-_-EOF-_-").await?;
        self.writer.flush().await?;

        Ok(())
    }
    // You have to asure to send the buffer afterwards
    pub async fn report_loading(&mut self) -> anyhow::Result<()> {
        self.dispatch("receiving", vec![]).await?;
        Ok(())
    }
    pub async fn send_code(&mut self, code: Vec<u8>) -> anyhow::Result<()> {
        sleep(Duration::from_millis(200)).await;
        self.dispatch("load", code).await?;

        // We need to wait for lua to read the buffer and be ready to the next one
        Ok(())
    }
}
