use crate::impl_into_packet;
pub use crate::proto::packets::*;

impl_into_packet!(RequestJoin, PacketId::REQUEST_JOIN);
impl_into_packet!(JoinResponse, PacketId::JOIN_RESPONSE);
impl_into_packet!(CreateBrowser<'a>, PacketId::CREATE_BROWSER);
impl_into_packet!(DestroyBrowser, PacketId::DESTROY_BROWSER);
impl_into_packet!(EmitEvent<'a>, PacketId::EMIT_EVENT);
