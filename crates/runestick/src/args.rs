/// Trait for converting arguments into values unsafely.
///
/// This has the ability to encode references.
pub trait Args {
    /// Encode arguments into a stack.
    ///
    /// # Safety
    ///
    /// This has the ability to encode references into the stack.
    /// The caller must ensure that the stack is cleared with
    /// [clear][Stack::clear] before the references are no longer valid.
    fn into_stack(self, stack: &mut crate::Stack) -> Result<(), crate::VmError>;

    /// Convert arguments into a vector.
    fn into_vec(self) -> Result<Vec<crate::Value>, crate::VmError>;

    /// The number of arguments.
    fn count() -> usize;
}

macro_rules! impl_into_args {
    () => {
        impl_into_args!{@impl 0,}
    };

    ({$ty:ident, $value:ident, $count:expr}, $({$l_ty:ident, $l_value:ident, $l_count:expr},)*) => {
        impl_into_args!{@impl $count, {$ty, $value, $count}, $({$l_ty, $l_value, $l_count},)*}
        impl_into_args!{$({$l_ty, $l_value, $l_count},)*}
    };

    (@impl $count:expr, $({$ty:ident, $value:ident, $ignore_count:expr},)*) => {
        impl<$($ty,)*> Args for ($($ty,)*)
        where
            $($ty: $crate::ToValue + std::fmt::Debug,)*
        {
            #[allow(unused)]
            fn into_stack(self, stack: &mut $crate::Stack) -> Result<(), $crate::VmError> {
                let ($($value,)*) = self;
                $(stack.push($value.to_value()?);)*
                Ok(())
            }

            #[allow(unused)]
            fn into_vec(self) -> Result<Vec<$crate::Value>, $crate::VmError> {
                let ($($value,)*) = self;
                $(let $value = <$ty>::to_value($value)?;)*
                Ok(vec![$($value,)*])
            }

            fn count() -> usize {
                $count
            }
        }
    };
}

repeat_macro!(impl_into_args);
