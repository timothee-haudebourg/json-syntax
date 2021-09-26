use smallvec::SmallVec;

pub use json_number::NumberBuf;

const SMALL_LEN: usize = 8;

pub enum Value {
    Null,
    Boolean(bool),
    Number(NumberBuf),
    Array(SmallVec<[Value; SMALL_LEN]>),
    Object(BTreeIndexMap)
}