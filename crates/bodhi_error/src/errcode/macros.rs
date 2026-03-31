//! 错误处理宏定义

#[macro_export]
macro_rules! define_error_codes {
  (
    $(
      $category:ident [$range_start:expr, $range_end:expr] {
        $(
          $($(#[doc = $doc:expr])+)?
          $name:ident = $code:expr
        ),+ $(,)?
      }
    )+
  ) => {
    $(
      pub mod $category {
        $(
          $crate::paste::paste! {
            $($(#[doc = $doc])+)?
            pub const [<$category:upper _ $name:upper>]: i32 = $code;
          }
        )+

        const _RANGE_CHECK: () = {
          $(
            ::core::assert!($code >= $range_start && $code <= $range_end,
              concat!(
                "Error code out of range for ",
                stringify!($category),
                ": ",
                stringify!($name),
                " should be in [", $range_start, ", ", $range_end, ")"
              )
            );
          )+
        };

        const _DUPLICATE_CHECK: () = {
          let codes: &[i32] = &[$($code),+];
            let mut i = 0usize;
            while i < codes.len() {
              let mut j = i + 1;
              while j < codes.len() {
                ::core::assert!(
                  codes[i] != codes[j],
                  concat!(
                    "Contains duplicate error codes in ",
                    stringify!($category)
                  )
                );
                j += 1;
              }
              i += 1;
            }
        };

        pub fn register() {
          $(
            $crate::errcode::register_error_code(
              $code,
              concat!(
                stringify!($category), "|",
                stringify!($name), "|",
                $($($doc),+)?
              )
            ).expect(
              concat!(
                "Failed to register error code for ",
                stringify!($category), "::", stringify!($name)));
          )+
        }
      }

      pub use $category::*;
    )+
  };
}
