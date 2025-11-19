#[macro_export]
macro_rules! const_assert_range {
  ($cond:expr, $msg:expr) => {
    const _: () = assert!($cond, $msg);
  };
}

#[macro_export]
macro_rules! define_static_errors {
  (
    $(
      $category:ident ($range_start:literal .. $range_end:literal) {
        $(
          $name:ident => ($code:literal, $msg:expr)
        ),+ $(,)?
      }
    )+
  ) => {
    $(
      pub mod $category {
        // use super::*;
        $(
          const _: () = {
            assert!(
              $code >= $range_start && $code < $range_end,
              concat!(
                "Error code out of range for ",
                stringify!($category),
                ": ",
                stringify!($name),
                " should be in [", $range_start, ", ", $range_end, ")"
              )
            );
          };

          $crate::paste::paste! {
            #[doc = concat!("Error metadata: `", stringify!($code), "`, `", $msg, "`")]
            #[allow(non_upper_case_globals)]
            pub static [<$category:upper _ $name:upper>]: $crate::ErrorMeta =
              $crate::ErrorMeta($code, $msg);
          }
        )+
      }

      pub use $category::*;
    )+
  };
}
