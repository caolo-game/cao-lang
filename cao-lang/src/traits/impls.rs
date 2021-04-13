use super::*;
use std::{alloc::Layout, convert::Infallible, mem};

impl<'a> ByteEncodeble for &'a str {
    fn displayname() -> &'static str {
        "Text"
    }
}

impl ByteEncodeble for str {
    fn displayname() -> &'static str {
        "Text"
    }
}

impl<'a> DecodeInPlace<'a> for str {
    type DecodeError = StringDecodeError;

    fn decode_in_place(bytes: &'a [u8]) -> Result<(usize, &'a str), StringDecodeError> {
        let (ll, len) = u32::decode_in_place(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = *len as usize;
        // if bytes.len() -
        let val = std::str::from_utf8(&bytes[ll..ll + len])
            .map_err(StringDecodeError::Utf8DecodeError)?;
        Ok((len + ll, val))
    }
}

impl<'a> ByteEncodeProperties for &'a str {
    type EncodeError = StringDecodeError;

    fn layout(&self) -> Layout {
        Layout::from_size_align(self.len() + std::mem::size_of::<u32>(), 4).unwrap()
    }
    fn encode(self, out: &mut [u8]) -> Result<(), Self::EncodeError> {
        if self.len() >= MAX_STR_LEN {
            return Err(StringDecodeError::LengthError(self.len()));
        }
        (self.len() as u32)
            .encode(out)
            .expect("failed to encode i32");
        unsafe {
            std::ptr::copy(self.as_ptr(), out.as_mut_ptr().add(4), self.len());
        }
        Ok(())
    }
}

impl ByteEncodeble for () {
    fn displayname() -> &'static str {
        "Void"
    }
}

impl ByteEncodeProperties for () {
    type EncodeError = Infallible;

    fn layout(&self) -> Layout {
        Layout::new::<()>()
    }

    fn encode(self, _out: &mut [u8]) -> Result<(), Infallible> {
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

impl<T: Sized + Clone + Copy + AutoByteEncodeProperties + std::fmt::Debug> ByteEncodeble for T {
    fn displayname() -> &'static str {
        <Self as AutoByteEncodeProperties>::displayname()
    }
}

// Types can't impl both Copy and Drop so we'll just encode using memcopy
impl<T: Sized + Copy + AutoByteEncodeProperties> ByteEncodeProperties for T {
    type EncodeError = Infallible;

    fn layout(&self) -> Layout {
        Layout::new::<Self>()
    }

    fn encode(self, out: &mut [u8]) -> Result<(), Infallible> {
        let ss = mem::size_of::<Self>();
        // TODO: maybe return error??? hmmmmm???
        assert!(ss <= out.len());
        unsafe {
            let ptr = out.as_mut_ptr();
            std::ptr::write_unaligned(ptr as *mut Self, self);
        }
        Ok(())
    }
}

// Types can't impl both Copy and Drop so we'll just decode using memcopy

impl<'a, T: Sized + Copy + AutoByteEncodeProperties + 'a> DecodeInPlace<'a> for T {
    type DecodeError = ();

    fn decode_in_place(bytes: &'a [u8]) -> Result<(usize, &'a T), Self::DecodeError> {
        let ss = mem::size_of::<Self>();
        if bytes.len() <= ss {
            Err(())
        } else {
            unsafe {
                let ptr = bytes.as_ptr() as *const Self;
                let result = &*ptr;
                Ok((ss, result))
            }
        }
    }
}
