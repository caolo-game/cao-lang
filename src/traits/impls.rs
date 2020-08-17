use super::*;
use std::convert::Infallible;

impl ByteEncodeProperties for String {
    const BYTELEN: usize = MAX_STR_LEN;

    type EncodeError = StringDecodeError;
    type DecodeError = StringDecodeError;

    fn displayname() -> &'static str {
        "Text"
    }

    fn encode(self) -> Result<Vec<u8>, Self::EncodeError> {
        if self.len() >= Self::BYTELEN {
            return Err(StringDecodeError::LengthError(self.len() as i32));
        }
        let mut rr = (self.len() as i32).encode().expect("failed to encode i32");
        rr.extend(self.as_bytes());
        Ok(rr)
    }

    fn decode(bytes: &[u8]) -> Result<Self, StringDecodeError> {
        let len = i32::decode(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = usize::try_from(len).map_err(|_| StringDecodeError::LengthError(len))?;
        std::str::from_utf8(&bytes[i32::BYTELEN..i32::BYTELEN + len])
            .map_err(|e| StringDecodeError::Utf8DecodeError(e))
            .map(|s| s.to_owned())
    }
}

impl ByteEncodeProperties for () {
    const BYTELEN: usize = 0;
    type EncodeError = Infallible;
    type DecodeError = Infallible;

    fn displayname() -> &'static str {
        "Void"
    }

    fn encode(self) -> Result<Vec<u8>, Infallible> {
        Ok(vec![])
    }

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

impl<T: Sized + Clone + Copy + AutoByteEncodeProperties + std::fmt::Debug> ByteEncodeProperties
    for T
{
    type EncodeError = Infallible;
    type DecodeError = ();

    fn encode(self) -> Result<Vec<u8>, Infallible> {
        let mut result = vec![0; Self::BYTELEN];
        unsafe {
            let dayum = mem::transmute::<*const Self, *const u8>(&self as *const Self);
            for i in 0..Self::BYTELEN {
                result[i] = *(dayum.add(i));
            }
        }
        Ok(result)
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> {
        if bytes.len() < Self::BYTELEN {
            Err(())
        } else {
            let result = unsafe { *(bytes.as_ptr() as *const Self) };
            Ok(result)
        }
    }
}
