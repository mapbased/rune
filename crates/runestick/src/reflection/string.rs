//! String trait implementations.

use crate::{
    FromValue, OwnedMut, OwnedRef, RawOwnedMut, RawOwnedRef, Shared, ToValue, UnsafeFromValue,
    Value, ValueError,
};

value_types!(crate::STRING_TYPE, String => String, &String, &mut String, &str, &mut str);

impl FromValue for String {
    fn from_value(value: Value) -> Result<Self, ValueError> {
        match value {
            Value::String(string) => Ok(string.borrow_ref()?.clone()),
            Value::StaticString(string) => Ok((**string).clone()),
            actual => Err(ValueError::expected::<String>(actual.type_info()?)),
        }
    }
}

impl ToValue for Box<str> {
    fn to_value(self) -> Result<Value, ValueError> {
        Ok(Value::String(Shared::new(self.to_string())))
    }
}

impl FromValue for Box<str> {
    fn from_value(value: Value) -> Result<Self, ValueError> {
        let string = value.into_string()?;
        let string = string.borrow_ref()?.clone();
        Ok(string.into_boxed_str())
    }
}

impl UnsafeFromValue for &str {
    type Output = *const str;
    type Guard = Option<RawOwnedRef>;

    unsafe fn unsafe_from_value(value: Value) -> Result<(Self::Output, Self::Guard), ValueError> {
        Ok(match value {
            Value::String(string) => {
                let string = string.owned_ref()?;
                let (s, guard) = OwnedRef::into_raw(string);
                ((*s).as_str(), Some(guard))
            }
            Value::StaticString(string) => (string.as_ref().as_str(), None),
            actual => return Err(ValueError::expected::<String>(actual.type_info()?)),
        })
    }

    unsafe fn to_arg(output: Self::Output) -> Self {
        &*output
    }
}

impl UnsafeFromValue for &mut str {
    type Output = *mut str;
    type Guard = Option<RawOwnedMut>;

    unsafe fn unsafe_from_value(value: Value) -> Result<(Self::Output, Self::Guard), ValueError> {
        Ok(match value {
            Value::String(string) => {
                let string = string.owned_mut()?;
                let (s, guard) = OwnedMut::into_raw(string);
                ((*s).as_mut_str(), Some(guard))
            }
            actual => {
                return Err(ValueError::expected::<String>(actual.type_info()?));
            }
        })
    }

    unsafe fn to_arg(output: Self::Output) -> Self {
        &mut *output
    }
}

impl UnsafeFromValue for &String {
    type Output = *const String;
    type Guard = Option<RawOwnedRef>;

    unsafe fn unsafe_from_value(value: Value) -> Result<(Self::Output, Self::Guard), ValueError> {
        Ok(match value {
            Value::String(string) => {
                let string = string.owned_ref()?;
                let (s, guard) = OwnedRef::into_raw(string);
                (s, Some(guard))
            }
            Value::StaticString(string) => (&**string, None),
            actual => {
                return Err(ValueError::expected::<String>(actual.type_info()?));
            }
        })
    }

    unsafe fn to_arg(output: Self::Output) -> Self {
        &*output
    }
}

impl UnsafeFromValue for &mut String {
    type Output = *mut String;
    type Guard = RawOwnedMut;

    unsafe fn unsafe_from_value(value: Value) -> Result<(Self::Output, Self::Guard), ValueError> {
        Ok(match value {
            Value::String(string) => {
                let string = string.owned_mut()?;
                let (s, guard) = OwnedMut::into_raw(string);
                (s, guard)
            }
            actual => {
                return Err(ValueError::expected::<String>(actual.type_info()?));
            }
        })
    }

    unsafe fn to_arg(output: Self::Output) -> Self {
        &mut *output
    }
}
