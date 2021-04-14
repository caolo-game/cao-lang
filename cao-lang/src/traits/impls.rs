use super::*;

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

impl ByteEncodeble for () {
    fn displayname() -> &'static str {
        "Void"
    }
}

impl ByteEncodeble for i8 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl ByteEncodeble for i16 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl ByteEncodeble for i32 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl ByteEncodeble for i64 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl ByteEncodeble for u8 {
    fn displayname() -> &'static str {
        "Unsigned Integer"
    }
}
impl ByteEncodeble for u16 {
    fn displayname() -> &'static str {
        "Unsigned Integer"
    }
}
impl ByteEncodeble for u32 {
    fn displayname() -> &'static str {
        "Unsigned Integer"
    }
}
impl ByteEncodeble for u64 {
    fn displayname() -> &'static str {
        "Unsigned Integer"
    }
}
impl ByteEncodeble for f32 {
    fn displayname() -> &'static str {
        "Floating point"
    }
}
impl ByteEncodeble for f64 {
    fn displayname() -> &'static str {
        "Floating point"
    }
}
