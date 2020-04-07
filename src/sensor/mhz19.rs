use std::io;
use std::path;
use std::time;

use bytes::{Buf, BufMut, BytesMut};
use futures::SinkExt;
use tokio::stream::StreamExt;
use tokio_serial::{Serial, SerialPortSettings};
use tokio_util::codec::{Decoder, Encoder, Framed};

pub struct MHZ19 {
    framed: Framed<Serial, Codec>,
}

impl MHZ19 {
    const BAUD_RATE: u32 = 9600;
    const TIMEOUT_MILLIS: u64 = 100;

    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<path::Path>,
    {
        let settings = SerialPortSettings {
            baud_rate: Self::BAUD_RATE,
            timeout: time::Duration::from_millis(Self::TIMEOUT_MILLIS),

            ..Default::default()
        };

        let port = Serial::from_path(path, &settings)?;
        let framed = Framed::new(port, Codec::default());

        Ok(MHZ19 { framed })
    }

    pub async fn measure(&mut self) -> Result<u32, io::Error> {
        log::trace!("start sending READ");
        self.framed.send(Command::Read).await?;
        log::trace!("finish sending READ");

        log::trace!("start reading");
        let r = match self.framed.next().await {
            Some(r) => r,
            None => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "empty read result",
            )),
        };
        log::trace!("finish reading");
        r
    }
}

enum Command {
    Read,
}

impl Command {
    const BYTES_READ: [u8; 9] = [0xff, 0x01, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79];

    fn into_bytes(self) -> &'static [u8] {
        match self {
            Self::Read => &Self::BYTES_READ,
        }
    }
}

#[derive(Default)]
struct Codec {}

impl Encoder<Command> for Codec {
    type Error = io::Error;

    fn encode(&mut self, item: Command, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_slice(item.into_bytes());
        Ok(())
    }
}

impl Decoder for Codec {
    type Item = u32;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let start_pos = src.as_ref().iter().position(|x| *x == 0xff);
        let start_pos = if let Some(p) = start_pos {
            p
        } else {
            log::trace!("starting byte is not found");
            return Ok(None);
        };

        src.advance(start_pos + 1);
        if src.len() < 8 {
            log::trace!("not enough data: {} < 8", src.len());
            return Ok(None);
        }

        let buf = src.split_to(8).to_vec();
        log::trace!("got data: {:?}", buf);
        // TODO: checksum
        if buf[0] == 0x86 {
            let high = buf[1] as u32;
            let low = buf[2] as u32;
            Ok(Some((high << 8) + low))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid data: {:?}", buf),
            ))
        }
    }
}
