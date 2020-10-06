use super::*;
use std::convert::Infallible;

impl<'a> ByteEncodeble for &'a str {
    const BYTELEN: usize = MAX_STR_LEN;

    fn displayname() -> &'static str {
        "Text"
    }
}

impl<'a> DecodeInPlace<'a> for &'a str {
    type Ref = Self;
    type DecodeError = StringDecodeError;

    fn decode_in_place(bytes: &'a [u8]) -> Result<Self::Ref, StringDecodeError> {
        let len = i32::decode(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = usize::try_from(len).map_err(|_| StringDecodeError::LengthError(len))?;
        std::str::from_utf8(&bytes[i32::BYTELEN..i32::BYTELEN + len])
            .map_err(|e| StringDecodeError::Utf8DecodeError(e))
    }
}

impl<'a> ByteEncodeProperties for &'a str {
    type EncodeError = StringDecodeError;

    fn encode(self, out: &mut Vec<u8>) -> Result<(), Self::EncodeError> {
        if self.len() >= Self::BYTELEN {
            return Err(StringDecodeError::LengthError(self.len() as i32));
        }
        (self.len() as i32)
            .encode(out)
            .expect("failed to encode i32");
        out.extend(self.as_bytes());
        Ok(())
    }
}

impl ByteEncodeble for String {
    const BYTELEN: usize = MAX_STR_LEN;

    fn displayname() -> &'static str {
        "Text"
    }
}

impl ByteEncodeProperties for String {
    type EncodeError = StringDecodeError;

    fn encode(self, out: &mut Vec<u8>) -> Result<(), Self::EncodeError> {
        if self.len() >= Self::BYTELEN {
            return Err(StringDecodeError::LengthError(self.len() as i32));
        }
        (self.len() as i32)
            .encode(out)
            .expect("failed to encode i32");
        out.extend(self.as_bytes());
        Ok(())
    }
}

impl ByteDecodeProperties for String {
    type DecodeError = StringDecodeError;

    fn decode(bytes: &[u8]) -> Result<Self, StringDecodeError> {
        let len = i32::decode(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = usize::try_from(len).map_err(|_| StringDecodeError::LengthError(len))?;
        std::str::from_utf8(&bytes[i32::BYTELEN..i32::BYTELEN + len])
            .map_err(|e| StringDecodeError::Utf8DecodeError(e))
            .map(|s| s.to_owned())
    }
}

impl ByteEncodeble for () {
    const BYTELEN: usize = 0;
    fn displayname() -> &'static str {
        "Void"
    }
}

impl ByteEncodeProperties for () {
    type EncodeError = Infallible;

    fn encode(self, _out: &mut Vec<u8>) -> Result<(), Infallible> {
        Ok(())
    }
}

impl ByteDecodeProperties for () {
    type DecodeError = Infallible;

    fn decode(_bytes: &[u8]) -> Result<Self, Self::DecodeError> {
        Ok(())
    }
}

impl AutoByteEncodeProperties for i8 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for i16 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for i32 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for i64 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u8 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u16 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u32 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u64 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for f32 {
    fn displayname() -> &'static str {
        "Floating point"
    }
}
impl AutoByteEncodeProperties for f64 {
    fn displayname() -> &'static str {
        "Floating point"
    }
}

impl<T1: AutoByteEncodeProperties> AutoByteEncodeProperties for (T1,) {}

impl<T1: AutoByteEncodeProperties, T2: AutoByteEncodeProperties> AutoByteEncodeProperties
    for (T1, T2)
{
}

impl<T1: AutoByteEncodeProperties, T2: AutoByteEncodeProperties, T3: AutoByteEncodeProperties>
    AutoByteEncodeProperties for (T1, T2, T3)
{
}

impl<
        T1: AutoByteEncodeProperties,
        T2: AutoByteEncodeProperties,
        T3: AutoByteEncodeProperties,
        T4: AutoByteEncodeProperties,
    > AutoByteEncodeProperties for (T1, T2, T3, T4)
{
}

impl<T: std::fmt::Debug> ObjectProperties for T {}

impl<T: Sized + Clone + Copy + AutoByteEncodeProperties + std::fmt::Debug> ByteEncodeble for T {
    const BYTELEN: usize = mem::size_of::<Self>();

    fn displayname() -> &'static str {
        <Self as AutoByteEncodeProperties>::displayname()
    }
}

impl<T: Sized + Clone + Copy + AutoByteEncodeProperties + std::fmt::Debug> ByteEncodeProperties
    for T
{
    type EncodeError = Infallible;

    fn encode(self, out: &mut Vec<u8>) -> Result<(), Infallible> {
        out.reserve(Self::BYTELEN);
        unsafe {
            let dayum = mem::transmute::<*const Self, *const u8>(&self as *const Self);
            for i in 0..Self::BYTELEN {
                out.push(*(dayum.add(i)));
            }
        }
        Ok(())
    }
}

impl<T: Sized + Clone + Copy + AutoByteEncodeProperties + std::fmt::Debug> ByteDecodeProperties
    for T
{
    type DecodeError = ();

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> {
        if bytes.len() < Self::BYTELEN {
            Err(())
        } else {
            let result = unsafe { *(bytes.as_ptr() as *const Self) };
            Ok(result)
        }
    }
}
