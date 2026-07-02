/// String stack capacity.
///
/// If a string is longer than this value,
/// it will be stored on the heap.
pub const JSON_STRING_STACK_CAPACITY: usize = 16;

/// String.
pub type JsonString = smallstr::SmallString<[u8; JSON_STRING_STACK_CAPACITY]>;
