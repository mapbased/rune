use core::cmp::Ordering;
use core::fmt;

use ::rust_alloc::sync::Arc;

use crate as rune;
use crate::alloc::clone::TryClone;
use crate::alloc::Box;

use super::{
    Accessor, FromValue, Mutable, OwnedRepr, OwnedTuple, ProtocolCaller, RuntimeError, Tuple,
    TypeInfo, Value, VariantRtti, Vec, VmResult,
};

/// The variant of a type.
#[derive(TryClone)]
pub struct Variant {
    pub(crate) rtti: Arc<VariantRtti>,
    pub(crate) data: VariantData,
}

impl Variant {
    /// Construct a field accessor from this variant.
    #[doc(hidden)]
    pub fn accessor<'a>(&'a self, data: &'a [Value]) -> Accessor<'a> {
        Accessor {
            fields: &self.rtti.fields,
            data,
        }
    }

    /// Try to access variant data as a tuple.
    pub fn as_tuple(&self) -> Option<&Tuple> {
        match &self.data {
            VariantData::Tuple(tuple) => Some(tuple),
            _ => None,
        }
    }

    /// Construct a unit variant.
    pub(crate) fn unit(rtti: Arc<VariantRtti>) -> Self {
        Self {
            rtti,
            data: VariantData::Empty,
        }
    }

    /// Construct a tuple variant.
    pub(crate) fn tuple(rtti: Arc<VariantRtti>, tuple: OwnedTuple) -> Self {
        Self {
            rtti,
            data: VariantData::Tuple(tuple),
        }
    }

    /// Construct a struct variant.
    pub(crate) fn struct_(rtti: Arc<VariantRtti>, data: Box<[Value]>) -> Self {
        Self {
            rtti,
            data: VariantData::Struct(data),
        }
    }

    /// Access the rtti of the variant.
    pub fn rtti(&self) -> &VariantRtti {
        &self.rtti
    }

    /// Access the underlying variant data.
    pub fn data(&self) -> &VariantData {
        &self.data
    }

    /// Access the underlying variant data mutably.
    pub(crate) fn data_mut(&mut self) -> &mut VariantData {
        &mut self.data
    }

    /// Get type info for the variant.
    pub(crate) fn type_info(&self) -> TypeInfo {
        TypeInfo::variant(self.rtti.clone())
    }

    pub(crate) fn partial_eq_with(
        a: &Self,
        b: &Self,
        caller: &mut dyn ProtocolCaller,
    ) -> VmResult<bool> {
        debug_assert_eq!(
            a.rtti.enum_hash, b.rtti.enum_hash,
            "comparison only makes sense if enum hashes match"
        );

        if a.rtti.hash != b.rtti.hash {
            return VmResult::Ok(false);
        }

        match (&a.data, &b.data) {
            (VariantData::Empty, VariantData::Empty) => VmResult::Ok(true),
            (VariantData::Tuple(a), VariantData::Tuple(b)) => {
                Vec::eq_with(a, b, Value::partial_eq_with, caller)
            }
            (VariantData::Struct(a), VariantData::Struct(b)) => {
                Vec::eq_with(a, b, Value::partial_eq_with, caller)
            }
            _ => VmResult::panic("data mismatch between variants"),
        }
    }

    pub(crate) fn eq_with(a: &Self, b: &Self, caller: &mut dyn ProtocolCaller) -> VmResult<bool> {
        debug_assert_eq!(
            a.rtti.enum_hash, b.rtti.enum_hash,
            "comparison only makes sense if enum hashes match"
        );

        if a.rtti.hash != b.rtti.hash {
            return VmResult::Ok(false);
        }

        match (&a.data, &b.data) {
            (VariantData::Empty, VariantData::Empty) => VmResult::Ok(true),
            (VariantData::Tuple(a), VariantData::Tuple(b)) => {
                Vec::eq_with(a, b, Value::eq_with, caller)
            }
            (VariantData::Struct(a), VariantData::Struct(b)) => {
                Vec::eq_with(a, b, Value::eq_with, caller)
            }
            _ => VmResult::panic("data mismatch between variants"),
        }
    }

    pub(crate) fn partial_cmp_with(
        a: &Self,
        b: &Self,
        caller: &mut dyn ProtocolCaller,
    ) -> VmResult<Option<Ordering>> {
        debug_assert_eq!(
            a.rtti.enum_hash, b.rtti.enum_hash,
            "comparison only makes sense if enum hashes match"
        );

        match a.rtti.hash.partial_cmp(&b.rtti.hash) {
            Some(Ordering::Equal) => {}
            ordering => return VmResult::Ok(ordering),
        }

        match (&a.data, &b.data) {
            (VariantData::Empty, VariantData::Empty) => VmResult::Ok(Some(Ordering::Equal)),
            (VariantData::Tuple(a), VariantData::Tuple(b)) => Vec::partial_cmp_with(a, b, caller),
            (VariantData::Struct(a), VariantData::Struct(b)) => Vec::partial_cmp_with(a, b, caller),
            _ => VmResult::panic("data mismatch between variants"),
        }
    }

    pub(crate) fn cmp_with(
        a: &Self,
        b: &Self,
        caller: &mut dyn ProtocolCaller,
    ) -> VmResult<Ordering> {
        debug_assert_eq!(
            a.rtti.enum_hash, b.rtti.enum_hash,
            "comparison only makes sense if enum hashes match"
        );

        match a.rtti.hash.cmp(&b.rtti.hash) {
            Ordering::Equal => {}
            ordering => return VmResult::Ok(ordering),
        }

        match (&a.data, &b.data) {
            (VariantData::Empty, VariantData::Empty) => VmResult::Ok(Ordering::Equal),
            (VariantData::Tuple(a), VariantData::Tuple(b)) => Vec::cmp_with(a, b, caller),
            (VariantData::Struct(a), VariantData::Struct(b)) => Vec::cmp_with(a, b, caller),
            _ => VmResult::panic("data mismatch between variants"),
        }
    }
}

impl FromValue for Variant {
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        match value.take_repr()? {
            OwnedRepr::Inline(value) => Err(RuntimeError::expected_variant(value.type_info())),
            OwnedRepr::Mutable(Mutable::Variant(value)) => Ok(value),
            OwnedRepr::Mutable(value) => Err(RuntimeError::expected_variant(value.type_info())),
            OwnedRepr::Any(value) => Err(RuntimeError::expected_variant(value.type_info())),
        }
    }
}

/// The data of the variant.
#[derive(TryClone)]
pub enum VariantData {
    /// A unit variant.
    Empty,
    /// A struct variant.
    Struct(Box<[Value]>),
    /// A tuple variant.
    Tuple(OwnedTuple),
}

impl fmt::Debug for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rtti.item)?;

        match &self.data {
            VariantData::Empty => {}
            VariantData::Struct(st) => {
                write!(f, "{:?}", st)?;
            }
            VariantData::Tuple(tuple) => {
                write!(f, "{:?}", tuple)?;
            }
        }

        Ok(())
    }
}
