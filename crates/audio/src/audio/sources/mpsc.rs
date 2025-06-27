//! Make a Tokio `Receiver<Bytes>` act as a `PacketSource`.

use bytes::Bytes;
use tokio::sync::mpsc::Receiver;

impl crate::audio::traits::PacketSource for Receiver<Bytes> {
    fn try_recv(&mut self) -> Option<Bytes> {
        Receiver::try_recv(self)
            .ok()
    }
}
