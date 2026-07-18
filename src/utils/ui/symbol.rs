// I can't use `.unwrap()` >:(
// But yeah, it works.

pub const ATTACHMENT_SYMBOL: &str = unsafe { str::from_utf8_unchecked(&[0xF0, 0x9F, 0x93, 0x8E]) };
pub const UNREAD_SYMBOL: &str = unsafe { str::from_utf8_unchecked(&[0xE2, 0x8F, 0xBA]) };
pub const CHECKMARK: &str = unsafe { str::from_utf8_unchecked(&[0xE2, 0x9C, 0x93]) };
