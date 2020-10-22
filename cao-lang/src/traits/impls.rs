use super::*;
use std::{convert::Infallible, mem};

impl<'a> DecodeInPlace<'a> for &'a str {
    type Ref = Self;
    type DecodeError = StringDecodeError;

    fn decode_in_place(bytes: &'a [u8]) -> Result<(usize, Self::Ref), StringDecodeError> {
        let (ll, len) = i32::decode(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = usize::try_from(len).map_err(|_| StringDecodeError::LengthError(len))?;
        let val = std::str::from_utf8(&bytes[ll..ll + len])
            .map_err(StringDecodeError::Utf8DecodeError)?;
        Ok((len + ll, val))
    }
}

impl<'a> ByteEncodeProperties for &'a str {
    type EncodeError = StringDecodeError;

    fn encode(self, out: &mut Vec<u8>) -> Result<(), Self::EncodeError> {
        if self.len() >= MAX_STR_LEN {
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
    fn displayname() -> &'static str {
        "Text"
    }
}

impl ByteEncodeProperties for String {
    type EncodeError = StringDecodeError;

    fn encode(self, out: &mut Vec<u8>) -> Result<(), Self::EncodeError> {
        if self.len() >= MAX_STR_LEN {
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

    fn decode(bytes: &[u8]) -> Result<(usize, Self), StringDecodeError> {
        let (ll, len) = i32::decode(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = usize::try_from(len).map_err(|_| StringDecodeError::LengthError(len))?;
        let tail = (ll + len).min(bytes.len());
        let res = std::str::from_utf8(&bytes[ll..tail])
            .map_err(StringDecodeError::Utf8DecodeError)
            .map(|s| s.to_owned())?;
        Ok((tail, res))
    }

    unsafe fn decode_unsafe(bytes: &[u8]) -> (usize, Self) {
        let (ll, len) = i32::decode_unsafe(bytes);
        let len = len as usize;
        let tail = (ll + len).min(bytes.len());
        let res = std::str::from_utf8(&bytes[ll..tail]).expect("Failed to decode utf8 string");
        (tail, res.to_owned())
    }
}

impl ByteEncodeble for () {
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

    fn decode(_bytes: &[u8]) -> Result<(usize, Self), Self::DecodeError> {
        Ok((0, ()))
    }

    unsafe fn decode_unsafe(_bytes: &[u8]) -> (usize, Self) {
        (0, ())
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
        "Unsigned Integer"
    }
}
impl AutoByteEncodeProperties for u16 {
    fn displayname() -> &'static str {
        "Unsigned Integer"
    }
}
impl AutoByteEncodeProperties for u32 {
    fn displayname() -> &'static str {
        "Unsigned Integer"
    }
}
impl AutoByteEncodeProperties for u64 {
    fn displayname() -> &'static str {
        "Unsigned Integer"
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
    fn displayname() -> &'static str {
        <Self as AutoByteEncodeProperties>::displayname()
    }
}

// Types can't impl both Copy and Drop so we'll just encode using memcopy
impl<T: Sized + AutoByteEncodeProperties> ByteEncodeProperties for T {
    type EncodeError = Infallible;

    fn encode(self, out: &mut Vec<u8>) -> Result<(), Infallible> {
        let ss = mem::size_of::<Self>();
        let ptr = out.len();
        out.resize(ptr + ss, 0);

        unsafe {
            let bytes = out.as_mut_ptr().add(ptr);
            let ptr = &mut *(bytes as *mut mem::MaybeUninit<Self>);
            *ptr.as_mut_ptr() = self;
        }
        Ok(())
    }
}

// Types can't impl both Copy and Drop so we'll just decode using memcopy
impl<T: Sized + AutoByteEncodeProperties> ByteDecodeProperties for T {
    type DecodeError = ();

    fn decode(bytes: &[u8]) -> Result<(usize, Self), Self::DecodeError> {
        let ss = mem::size_of::<Self>();
        if bytes.len() < ss {
            Err(())
        } else {
            let result = unsafe { *(bytes.as_ptr() as *const Self) };
            Ok((ss, result))
        }
    }

    unsafe fn decode_unsafe(bytes: &[u8]) -> (usize, Self) {
        let ss = mem::size_of::<Self>();
        let result = *(bytes.as_ptr() as *const Self);
        (ss, result)
    }
}

impl<'a, T: Sized + AutoByteEncodeProperties + 'a> DecodeInPlace<'a> for T {
    type Ref = &'a Self;

    type DecodeError = ();

    fn decode_in_place(bytes: &'a [u8]) -> Result<(usize, Self::Ref), Self::DecodeError> {
        let ss = mem::size_of::<Self>();
        if bytes.len() < ss {
            Err(())
        } else {
            let result = unsafe { &*(bytes.as_ptr() as *const Self) };
            Ok((ss, result))
        }
    }
}
