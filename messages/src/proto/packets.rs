// Automatically generated rust module for 'packets.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PacketId {
    OPEN_CONNECTION = 0,
    REQUEST_JOIN = 1,
    JOIN_RESPONSE = 2,
    CREATE_BROWSER = 3,
    DESTROY_BROWSER = 4,
    ALWAYS_LISTEN_KEYS = 5,
    HIDE_BROWSER = 6,
    FOCUS_BROWSER = 7,
    CREATE_EXTERNAL_BROWSER = 11,
    APPEND_TO_OBJECT = 12,
    REMOVE_FROM_OBJECT = 13,
    TOGGLE_DEV_TOOLS = 14,
    SET_AUDIO_SETTINGS = 15,
    LOAD_URL = 16,
    EMIT_EVENT = 8,
    BROWSER_CREATED = 9,
    GOT = 10,
}

impl Default for PacketId {
    fn default() -> Self {
        PacketId::OPEN_CONNECTION
    }
}

impl From<i32> for PacketId {
    fn from(i: i32) -> Self {
        match i {
            0 => PacketId::OPEN_CONNECTION,
            1 => PacketId::REQUEST_JOIN,
            2 => PacketId::JOIN_RESPONSE,
            3 => PacketId::CREATE_BROWSER,
            4 => PacketId::DESTROY_BROWSER,
            5 => PacketId::ALWAYS_LISTEN_KEYS,
            6 => PacketId::HIDE_BROWSER,
            7 => PacketId::FOCUS_BROWSER,
            11 => PacketId::CREATE_EXTERNAL_BROWSER,
            12 => PacketId::APPEND_TO_OBJECT,
            13 => PacketId::REMOVE_FROM_OBJECT,
            14 => PacketId::TOGGLE_DEV_TOOLS,
            15 => PacketId::SET_AUDIO_SETTINGS,
            16 => PacketId::LOAD_URL,
            8 => PacketId::EMIT_EVENT,
            9 => PacketId::BROWSER_CREATED,
            10 => PacketId::GOT,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for PacketId {
    fn from(s: &'a str) -> Self {
        match s {
            "OPEN_CONNECTION" => PacketId::OPEN_CONNECTION,
            "REQUEST_JOIN" => PacketId::REQUEST_JOIN,
            "JOIN_RESPONSE" => PacketId::JOIN_RESPONSE,
            "CREATE_BROWSER" => PacketId::CREATE_BROWSER,
            "DESTROY_BROWSER" => PacketId::DESTROY_BROWSER,
            "ALWAYS_LISTEN_KEYS" => PacketId::ALWAYS_LISTEN_KEYS,
            "HIDE_BROWSER" => PacketId::HIDE_BROWSER,
            "FOCUS_BROWSER" => PacketId::FOCUS_BROWSER,
            "CREATE_EXTERNAL_BROWSER" => PacketId::CREATE_EXTERNAL_BROWSER,
            "APPEND_TO_OBJECT" => PacketId::APPEND_TO_OBJECT,
            "REMOVE_FROM_OBJECT" => PacketId::REMOVE_FROM_OBJECT,
            "TOGGLE_DEV_TOOLS" => PacketId::TOGGLE_DEV_TOOLS,
            "SET_AUDIO_SETTINGS" => PacketId::SET_AUDIO_SETTINGS,
            "LOAD_URL" => PacketId::LOAD_URL,
            "EMIT_EVENT" => PacketId::EMIT_EVENT,
            "BROWSER_CREATED" => PacketId::BROWSER_CREATED,
            "GOT" => PacketId::GOT,
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

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
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

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
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

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_bool(*&self.success))?;
        if let Some(ref s) = self.current_version { w.write_with_tag(16, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CreateBrowser<'a> {
    pub browser_id: u32,
    pub url: Cow<'a, str>,
    pub hidden: bool,
    pub focused: bool,
}

impl<'a> MessageRead<'a> for CreateBrowser<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(18) => msg.url = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(24) => msg.hidden = r.read_bool(bytes)?,
                Ok(32) => msg.focused = r.read_bool(bytes)?,
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
        + 1 + sizeof_varint(*(&self.hidden) as u64)
        + 1 + sizeof_varint(*(&self.focused) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.url))?;
        w.write_with_tag(24, |w| w.write_bool(*&self.hidden))?;
        w.write_with_tag(32, |w| w.write_bool(*&self.focused))?;
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

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct AlwaysListenKeys {
    pub browser_id: u32,
    pub listen: bool,
}

impl<'a> MessageRead<'a> for AlwaysListenKeys {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(16) => msg.listen = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for AlwaysListenKeys {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_varint(*(&self.listen) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(16, |w| w.write_bool(*&self.listen))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct EmitEvent<'a> {
    pub event_name: Cow<'a, str>,
    pub args: Option<Cow<'a, str>>,
    pub arguments: Vec<EventValue<'a>>,
}

impl<'a> MessageRead<'a> for EmitEvent<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.event_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.args = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(26) => msg.arguments.push(r.read_message::<EventValue>(bytes)?),
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
        + self.args.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.arguments.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.event_name))?;
        if let Some(ref s) = self.args { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        for s in &self.arguments { w.write_with_tag(26, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct HideBrowser {
    pub browser_id: u32,
    pub hide: bool,
}

impl<'a> MessageRead<'a> for HideBrowser {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(16) => msg.hide = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HideBrowser {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_varint(*(&self.hide) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(16, |w| w.write_bool(*&self.hide))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct FocusBrowser {
    pub browser_id: u32,
    pub focused: bool,
}

impl<'a> MessageRead<'a> for FocusBrowser {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(16) => msg.focused = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for FocusBrowser {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_varint(*(&self.focused) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(16, |w| w.write_bool(*&self.focused))?;
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

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.string_value { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.float_value { w.write_with_tag(21, |w| w.write_float(*s))?; }
        if let Some(ref s) = self.integer_value { w.write_with_tag(24, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct BrowserCreated {
    pub browser_id: u32,
    pub status_code: i32,
}

impl<'a> MessageRead<'a> for BrowserCreated {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(16) => msg.status_code = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for BrowserCreated {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_varint(*(&self.status_code) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.status_code))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Got { }

impl<'a> MessageRead<'a> for Got {
    fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for Got { }

#[derive(Debug, Default, PartialEq, Clone)]
pub struct OpenConnection { }

impl<'a> MessageRead<'a> for OpenConnection {
    fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for OpenConnection { }

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CreateExternalBrowser<'a> {
    pub browser_id: u32,
    pub url: Cow<'a, str>,
    pub scale: i32,
    pub texture: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for CreateExternalBrowser<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(18) => msg.url = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(24) => msg.scale = r.read_int32(bytes)?,
                Ok(34) => msg.texture = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for CreateExternalBrowser<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_len((&self.url).len())
        + 1 + sizeof_varint(*(&self.scale) as u64)
        + 1 + sizeof_len((&self.texture).len())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.url))?;
        w.write_with_tag(24, |w| w.write_int32(*&self.scale))?;
        w.write_with_tag(34, |w| w.write_string(&**&self.texture))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct AppendToObject {
    pub browser_id: u32,
    pub object_id: i32,
}

impl<'a> MessageRead<'a> for AppendToObject {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(16) => msg.object_id = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for AppendToObject {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_varint(*(&self.object_id) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.object_id))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct RemoveFromObject {
    pub browser_id: u32,
    pub object_id: i32,
}

impl<'a> MessageRead<'a> for RemoveFromObject {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(16) => msg.object_id = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for RemoveFromObject {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_varint(*(&self.object_id) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.object_id))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ToggleDevTools {
    pub browser_id: u32,
    pub enabled: bool,
}

impl<'a> MessageRead<'a> for ToggleDevTools {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(16) => msg.enabled = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ToggleDevTools {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_varint(*(&self.enabled) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(16, |w| w.write_bool(*&self.enabled))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SetAudioSettings {
    pub browser_id: u32,
    pub max_distance: f32,
    pub reference_distance: f32,
}

impl<'a> MessageRead<'a> for SetAudioSettings {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.browser_id = r.read_uint32(bytes)?,
                Ok(21) => msg.max_distance = r.read_float(bytes)?,
                Ok(29) => msg.reference_distance = r.read_float(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for SetAudioSettings {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + 4
        + 1 + 4
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(21, |w| w.write_float(*&self.max_distance))?;
        w.write_with_tag(29, |w| w.write_float(*&self.reference_distance))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct LoadUrl<'a> {
    pub browser_id: u32,
    pub url: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for LoadUrl<'a> {
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

impl<'a> MessageWrite for LoadUrl<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.browser_id) as u64)
        + 1 + sizeof_len((&self.url).len())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.browser_id))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.url))?;
        Ok(())
    }
}

