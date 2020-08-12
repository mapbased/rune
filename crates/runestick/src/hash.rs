use crate::value::ValueType;
use std::fmt;
use std::hash::{BuildHasher as _, BuildHasherDefault, Hash as _, Hasher as _};
use twox_hash::XxHash64;

/// The hash of a primitive thing.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash(pub(crate) u64);

impl Hash {
    /// Hash corresponding to global function calls.
    pub const GLOBAL_MODULE: Hash = Hash(0);

    const SEP: usize = 0x7f;
    const TYPE: usize = 1;
    const INSTANCE_FUNCTION: usize = 3;
    const OBJECT_KEYS: usize = 4;
    const TUPLE_MATCH: usize = 5;

    /// Construct a simple hash from something that is hashable.
    pub fn of<T: std::hash::Hash>(thing: T) -> Self {
        let mut hasher = BuildHasherDefault::<XxHash64>::default().build_hasher();
        thing.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Hash the given iterator of object keys.
    pub fn object_keys<I>(keys: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut hasher = BuildHasherDefault::<XxHash64>::default().build_hasher();
        Self::OBJECT_KEYS.hash(&mut hasher);

        for key in keys {
            Self::SEP.hash(&mut hasher);
            key.as_ref().hash(&mut hasher);
        }

        Self(hasher.finish())
    }

    /// Construct a hash for an use.
    fn path<I>(kind: usize, path: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut hasher = BuildHasherDefault::<XxHash64>::default().build_hasher();
        kind.hash(&mut hasher);

        for part in path {
            part.as_ref().hash(&mut hasher);
            Self::SEP.hash(&mut hasher);
        }

        Self(hasher.finish())
    }

    /// Get the hash of a type.
    pub fn of_type<I>(path: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::path(Self::TYPE, path)
    }

    /// Get the hash of a tuple match function.
    pub fn tuple_match<I>(path: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::path(Self::TUPLE_MATCH, path)
    }

    /// Construct a hash for a function in the given path.
    pub fn function<I>(path: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::path(Self::TYPE, path)
    }

    /// Construct a hash to an instance function, where the instance is a
    /// pre-determined type.
    pub fn instance_function(ty: ValueType, name: Hash) -> Self {
        Self::of((Self::INSTANCE_FUNCTION, ty, Self::SEP, name))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "0x{:x}", self.0)
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "Hash(0x{:x})", self.0)
    }
}