
use std::{io, rc::Rc};

use crate::bytes::{Bytes, BytesMut, BytesVec};


pub trait EncoderDecoder:Encoder+Decoder{}

impl <T>EncoderDecoder for T where T:Encoder+Decoder{}

/// Trait of helper objects to write out messages as bytes.
pub trait Encoder {
    /// The type of items consumed by the `Encoder`
    type EncodeItem;

    /// The type of encoding errors.
    type EncodeError: std::fmt::Debug;

    /// Encodes a frame into the buffer provided.
    fn encode(&self, item: Self::EncodeItem, dst: &mut BytesMut) -> Result<(), Self::EncodeError>;

    /// Encodes a frame into the buffer provided.
    fn encode_vec(&self, item: Self::EncodeItem, dst: &mut BytesVec) -> Result<(), Self::EncodeError> {
        dst.with_bytes_mut(|dst| self.encode(item, dst))
    }
}

/// Decoding of frames via buffers.
pub trait Decoder {
    /// The type of decoded frames.
    type DecodeItem;

    /// The type of unrecoverable frame decoding errors.
    ///
    /// If an individual message is ill-formed but can be ignored without
    /// interfering with the processing of future messages, it may be more
    /// useful to report the failure as an `Item`.
    type DecodeError: std::fmt::Debug;

    /// Attempts to decode a frame from the provided buffer of bytes.
    fn decode(&self, src: &mut BytesMut) -> Result<Option<Self::DecodeItem>, Self::DecodeError>;

    /// Attempts to decode a frame from the provided buffer of bytes.
    fn decode_vec(&self, src: &mut BytesVec) -> Result<Option<Self::DecodeItem>, Self::DecodeError> {
        src.with_bytes_mut(|src| self.decode(src))
    }
}

impl<T> Encoder for Rc<T>
where
    T: Encoder,
{
    type EncodeItem = T::EncodeItem;
    type EncodeError = T::EncodeError;

    fn encode(&self, item: Self::EncodeItem, dst: &mut BytesMut) -> Result<(), Self::EncodeError> {
        (**self).encode(item, dst)
    }
}

impl<T> Decoder for Rc<T>
where
    T: Decoder,
{
    type DecodeItem = T::DecodeItem;
    type DecodeError = T::DecodeError;

    fn decode(&self, src: &mut BytesMut) -> Result<Option<Self::DecodeItem>, Self::DecodeError> {
        (**self).decode(src)
    }
}

/// Bytes codec.
///
/// Reads/Writes chunks of bytes from a stream.
#[derive(Debug, Copy, Clone)]
pub struct BytesCodec;

impl Encoder for BytesCodec {
    type EncodeItem = Bytes;
    type EncodeError = io::Error;

    #[inline]
    fn encode(&self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::EncodeError> {
        dst.extend_from_slice(&item[..]);
        Ok(())
    }
}

impl Decoder for BytesCodec {
    type DecodeItem = BytesMut;
    type DecodeError = io::Error;

    fn decode(&self, src: &mut BytesMut) -> Result<Option<Self::DecodeItem>, Self::DecodeError> {
        if src.is_empty() {
            Ok(None)
        } else {
            let len = src.len();
            Ok(Some(src.split_to(len)))
        }
    }
}
