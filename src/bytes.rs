use smallvec::SmallVec;

/// Buffer type.
///
/// # Safety
///
/// The `AsRef<[u8]>` implementation *must* return the bytes provided using
/// the `from_bytes` and `from_vec` constructor functions.
pub unsafe trait JsonBytes: AsRef<[u8]> {
	fn from_bytes(bytes: &[u8]) -> Self;

	fn from_vec(bytes: Vec<u8>) -> Self;
}

unsafe impl JsonBytes for Vec<u8> {
	fn from_bytes(bytes: &[u8]) -> Self {
		bytes.into()
	}

	fn from_vec(bytes: Vec<u8>) -> Self {
		bytes
	}
}

unsafe impl<A: smallvec::Array<Item = u8>> JsonBytes for SmallVec<A> {
	fn from_vec(bytes: Vec<u8>) -> Self {
		bytes.into()
	}

	fn from_bytes(bytes: &[u8]) -> Self {
		bytes.into()
	}
}
