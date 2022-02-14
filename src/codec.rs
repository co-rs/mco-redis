
use std::{io, rc::Rc};

use crate::bytes::{Bytes, BytesMut, BytesVec};


pub trait EncoderDecoder:Encoder+Decoder{}

impl <T>EncoderDecoder for T where T:Encoder+Decoder{}

/// Trait of helper objects to write out messages as bytes.
pub trait Encoder {
    /// The type of items consumed by the `Encoder`
    type ItemEncode;

    /// The type of encoding errors.
    type Error: std::fmt::Debug;

    /// Encodes a frame into the buffer provided.
    fn encode(&self, item: Self::ItemEncode, dst: &mut BytesMut) -> Result<(), Self::Error>;

    /// Encodes a frame into the buffer provided.
    fn encode_vec(&self, item: Self::ItemEncode, dst: &mut BytesVec) -> Result<(), Self::Error> {
        dst.with_bytes_mut(|dst| self.encode(item, dst))
    }
}

/// Decoding of frames via buffers.
pub trait Decoder {
    /// The type of decoded frames.
    type ItemDecode;

    /// The type of unrecoverable frame decoding errors.
    ///
    /// If an individual message is ill-formed but can be ignored without
    /// interfering with the processing of future messages, it may be more
    /// useful to report the failure as an `Item`.
    type Error: std::fmt::Debug;

    /// Attempts to decode a frame from the provided buffer of bytes.
    fn decode(&self, src: &mut BytesMut) -> Result<Option<Self::ItemDecode>, Self::Error>;

    /// Attempts to decode a frame from the provided buffer of bytes.
    fn decode_vec(&self, src: &mut BytesVec) -> Result<Option<Self::ItemDecode>, Self::Error> {
        src.with_bytes_mut(|src| self.decode(src))
    }
}

impl<T> Encoder for Rc<T>
where
    T: Encoder,
{
    type ItemEncode = T::ItemEncode;
    type Error = T::Error;

    fn encode(&self, item: Self::ItemEncode, dst: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode(item, dst)
    }
}

impl<T> Decoder for Rc<T>
where
    T: Decoder,
{
    type ItemDecode = T::ItemDecode;
    type Error = T::Error;

    fn decode(&self, src: &mut BytesMut) -> Result<Option<Self::ItemDecode>, Self::Error> {
        (**self).decode(src)
    }
}

/// Bytes codec.
///
/// Reads/Writes chunks of bytes from a stream.
#[derive(Debug, Copy, Clone)]
pub struct BytesCodec;

impl Encoder for BytesCodec {
    type ItemEncode = Bytes;
    type Error = io::Error;

    #[inline]
    fn encode(&self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(&item[..]);
        Ok(())
    }
}

impl Decoder for BytesCodec {
    type ItemDecode = BytesMut;
    type Error = io::Error;

    fn decode(&self, src: &mut BytesMut) -> Result<Option<Self::ItemDecode>, Self::Error> {
        if src.is_empty() {
            Ok(None)
        } else {
            let len = src.len();
            Ok(Some(src.split_to(len)))
        }
    }
}
