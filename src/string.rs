/// String stack capacity.
///
/// If a string is longer than this value,
/// it will be stored on the heap.
pub const SMALL_STRING_CAPACITY: usize = 16;

/// String.
pub type JsonString = smallstr::SmallString<[u8; SMALL_STRING_CAPACITY]>;
