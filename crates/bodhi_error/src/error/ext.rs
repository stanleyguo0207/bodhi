//! 错误扩展模块

use std::result::Result as StdResult;

use super::{Error, Result};

/// 结果扩展
pub trait ResultExt<T> {
  /// 追加上下文信息
  ///
  /// 用于在错误发生时静态生成上下文信息。
  ///
  /// # Examples
  ///
  /// ```
  /// let result = Result::<(), Error>::Ok(());
  /// let result = result.wrap_context("static context".to_string());
  /// ```
  fn wrap_context(self, msg: impl Into<String>) -> Result<T>;
  /// 延迟追加上下文信息
  ///
  /// 用于在错误发生时动态生成上下文信息。
  ///
  /// # Examples
  ///
  /// ```
  /// let result = Result::<(), Error>::Ok(());
  /// let result = result.wrap_context_with(|| "dynamic context".to_string());
  /// ```
  fn wrap_context_with<F: FnOnce() -> String>(self, f: F) -> Result<T>;
}

impl<T> ResultExt<T> for StdResult<T, Error> {
  fn wrap_context(self, msg: impl Into<String>) -> Result<T> {
    self.map_err(|e| e.wrap_context(msg))
  }

  fn wrap_context_with<F: FnOnce() -> String>(self, f: F) -> Result<T> {
    self.map_err(|e| e.wrap_context_with(f))
  }
}

/// 选项扩展
pub trait OptionExt<T> {
  /// 将选项转换为结果
  ///
  /// 用于将选项转换为结果，如果选项为 None，则返回错误。
  ///
  /// # Examples
  ///
  /// ```
  /// let option = Option::<i32>::None;
  /// let result = option.ok_or_err(1);
  /// ```
  fn ok_or_err(self, code: i32) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
  fn ok_or_err(self, code: i32) -> Result<T> {
    self.ok_or_else(|| Error::new(code))
  }
}
