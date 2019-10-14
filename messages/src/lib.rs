use quick_protobuf::serialize_into_vec;
use std::convert::TryInto;

pub mod packets;
pub mod proto;

#[macro_export]
macro_rules! impl_into_packet {
    ($type:ty, $id:path) => {
        impl<'a> std::convert::TryFrom<$type> for Packet<'a> {
            type Error = quick_protobuf::Error;

            fn try_from(packet: $type) -> Result<Self, Self::Error> {
                Ok(crate::packets::Packet {
                    packet_id: $id,
                    bytes: std::borrow::Cow::Owned(quick_protobuf::serialize_into_vec(&packet)?),
                })
            }
        }
    };
}

pub fn try_into_packet<'a, T>(value: T) -> Result<Vec<u8>, quick_protobuf::Error>
where
    T: TryInto<packets::Packet<'a>, Error = quick_protobuf::Error>,
{
    T::try_into(value).and_then(|packet| serialize_into_vec(&packet))
}
