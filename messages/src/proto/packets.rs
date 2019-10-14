// Automatically generated rust module for 'packets.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::io::Write;
use std::borrow::Cow;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PacketId {
    REQUEST_JOIN = 1,
    JOIN_RESPONSE = 2,
    CREATE_BROWSER = 3,
    DESTROY_BROWSER = 4,
    EMIT_EVENT = 5,
}

impl Default for PacketId {
    fn default() -> Self {
        PacketId::REQUEST_JOIN
    }
}

impl From<i32> for PacketId {
    fn from(i: i32) -> Self {
        match i {
            1 => PacketId::REQUEST_JOIN,
            2 => PacketId::JOIN_RESPONSE,
            3 => PacketId::CREATE_BROWSER,
            4 => PacketId::DESTROY_BROWSER,
            5 => PacketId::EMIT_EVENT,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for PacketId {
    fn from(s: &'a str) -> Self {
        match s {
            "REQUEST_JOIN" => PacketId::REQUEST_JOIN,
            "JOIN_RESPONSE" => PacketId::JOIN_RESPONSE,
            "CREATE_BROWSER" => PacketId::CREATE_BROWSER,
            "DESTROY_BROWSER" => PacketId::DESTROY_BROWSER,
            "EMIT_EVENT" => PacketId::EMIT_EVENT,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Packet<'a> {
    pub packet_id: PacketId,
    pub bytes: Cow<'a, [u8]>,
}

impl<'a> MessageRead<'a> for Packet<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.packet_id = r.read_enum(bytes)?,
                Ok(18) => msg.bytes = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Packet<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.packet_id) as u64)
        + 1 + sizeof_len((&self.bytes).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_enum(*&self.packet_id as i32))?;
        w.write_with_tag(18, |w| w.write_bytes(&**&self.bytes))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct RequestJoin {
    pub plugin_version: i32,
}

impl<'a> MessageRead<'a> for RequestJoin {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.plugin_version = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for RequestJoin {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.plugin_version) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int32(*&self.plugin_version))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct JoinResponse {
    pub success: bool,
    pub current_version: Option<i32>,
}

impl<'a> MessageRead<'a> for JoinResponse {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.success = r.read_bool(bytes)?,
                Ok(16) => msg.current_version = Some(r.read_int32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for JoinResponse {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.success) as u64)
        + self.current_version.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_bool(*&self.success))?;
        if let Some(ref s) = self.current_version { w.write_with_tag(16, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CreateBrowser<'a> {
    pub browser_id: u32,
    pub url: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for CreateBrowser<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(18) => msg.url = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for CreateBrowser<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_len((&self.url).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.url))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct DestroyBrowser {
    pub browser_id: u32,
}

impl<'a> MessageRead<'a> for DestroyBrowser {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for DestroyBrowser {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct EmitEvent<'a> {
    pub event_name: Cow<'a, str>,
    pub arguments: Vec<EventValue<'a>>,
}

impl<'a> MessageRead<'a> for EmitEvent<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.arguments.push(r.read_message::<EventValue>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for EmitEvent<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.event_name).len())
        + self.arguments.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.event_name))?;
        for s in &self.arguments { w.write_with_tag(18, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct EventValue<'a> {
    pub string_value: Option<Cow<'a, str>>,
    pub float_value: Option<f32>,
    pub integer_value: Option<i32>,
}

impl<'a> MessageRead<'a> for EventValue<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.string_value = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(21) => msg.float_value = Some(r.read_float(bytes)?),
                Ok(24) => msg.integer_value = Some(r.read_int32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for EventValue<'a> {
    fn get_size(&self) -> usize {
        0
        + self.string_value.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.float_value.as_ref().map_or(0, |_| 1 + 4)
        + self.integer_value.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.string_value { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.float_value { w.write_with_tag(21, |w| w.write_float(*s))?; }
        if let Some(ref s) = self.integer_value { w.write_with_tag(24, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

