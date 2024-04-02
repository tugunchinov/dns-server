mod enums;
mod header;
mod packet;
mod packet_builder;
mod question;
mod record;

pub use enums::*;
pub use packet::*;
pub use packet_builder::{DnsPacketBuilder, RawRecordType};
pub use question::Question;

pub fn new_packet_buffer() -> Vec<u8> {
    vec![0u8; 512]
}
