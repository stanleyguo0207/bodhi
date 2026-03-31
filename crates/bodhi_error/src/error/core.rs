//! 核心错误类型定义模块

use backtrace::Backtrace;

use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};

use super::{Frame, apply_frames_filters};
use crate::errcode::BODHIERR_SYS;

/// 核心错误类型
pub struct Error {
  /// 错误码 （0=成功， <0=框架错误， >0=业务错误）
  code: i32,
  /// 源错误
  source: Option<Box<dyn StdError + Send + Sync + 'static>>,
  /// 上下文
  contexts: Option<Vec<String>>,
  /// 调用栈
  backtrace: Option<Backtrace>,
}

impl Error {
  /// 创建指定错误码的错误
  ///
  /// 用于创建指定错误码的错误。
  ///
  /// # Examples
  ///
  /// ```
  /// let error = Error::new(1);
  /// ```
  pub fn new(code: i32) -> Self {
    Self {
      code,
      source: None,
      contexts: None,
      backtrace: None,
    }
    .capture_backtrace()
  }

  /// 从任意标准错误创建错误
  ///
  /// 用于从任意标准错误创建错误。
  ///
  /// # Examples
  ///
  /// ```
  /// let error = Error::from_std(std::io::Error::new(std::io::ErrorKind::Other, "io error"));
  /// ```
  pub fn from_std<E>(error: E) -> Self
  where
    E: StdError + Send + Sync + 'static,
  {
    Self {
      code: BODHIERR_SYS,
      source: Some(Box::new(error)),
      contexts: None,
      backtrace: None,
    }
    .capture_backtrace()
  }

  /// 追加上下文信息
  ///
  /// 用于在错误发生时追加上下文信息。
  ///
  /// # Examples
  ///
  /// ```
  /// let error = Error::new(1).wrap_context("context message");
  /// ```
  pub fn wrap_context(mut self, msg: impl Into<String>) -> Self {
    self.contexts.get_or_insert_with(Vec::new).push(msg.into());
    self
  }

  /// 延迟追加上下文信息
  ///
  /// 用于在错误发生时动态追加上下文信息。
  ///
  /// # Examples
  ///
  /// ```
  /// let error = Error::new(1).wrap_context_with(|| "context message".to_string());
  /// ```
  pub fn wrap_context_with<F: FnOnce() -> String>(mut self, f: F) -> Self {
    self.contexts.get_or_insert_with(Vec::new).push(f());
    self
  }

  /// 获取错误码
  pub fn code(&self) -> i32 {
    self.code
  }

  /// 获取调用栈
  fn capture_backtrace(mut self) -> Self {
    self.backtrace = Some(Backtrace::new());
    self
  }

  /// 生成源错误摘要信息
  ///
  /// 用于 `Display` 输出，最多显示前 5 条源错误。
  fn source_summary(&self) -> String {
    if let Some(source) = &self.source {
      let max_items = 5;
      let mut chain = Vec::with_capacity(max_items);
      let mut total = 0;

      // 遍历整个错误链，收集所有错误信息
      let mut current: Option<&dyn StdError> = Some(source.as_ref());
      while let Some(cause) = current {
        total += 1;
        if chain.len() < max_items {
          chain.push(format!("{}", cause));
        }
        current = cause.source();
      }

      let summary = chain.join(" -> ");

      match (chain.len(), total) {
        (0, 0) => String::from("None"),
        (_, total) if total <= max_items => summary,
        (_, total) => format!("{} ...({} total)", summary, total),
      }
    } else {
      String::from("None")
    }
  }

  /// 生成上下文摘要信息
  ///
  /// 用于 `Display` 输出，最多显示前 5 条上下文信息。
  fn context_summary(&self) -> String {
    if let Some(contexts) = &self.contexts {
      let max_items = 5;

      let shown: Vec<&str> = contexts
        .iter()
        .take(max_items)
        .map(|s| s.as_str())
        .collect();
      let summary = shown.join(" -> ");
      if contexts.len() > max_items {
        format!("{} ...({} total)", summary, contexts.len())
      } else {
        summary
      }
    } else {
      String::from("None")
    }
  }
}

impl Debug for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Meta:")?;
    writeln!(f, "  code: {}", self.code)?;

    if let Some(source) = &self.source {
      writeln!(f, "Source:")?;
      let mut current: Option<&dyn StdError> = Some(source.as_ref());
      let mut depth = 1;
      while let Some(err) = current {
        writeln!(f, "  [{depth}] {err}")?;
        current = err.source();
        depth += 1;
      }
    }

    if let Some(contexts) = &self.contexts {
      writeln!(f, "Contexts:")?;
      for (i, ctx) in contexts.iter().enumerate() {
        writeln!(f, "  [{}] {}", i + 1, ctx)?;
      }
    }

    if let Some(backtrace) = &self.backtrace {
      let frames: Vec<_> = backtrace
        .frames()
        .iter()
        .flat_map(|frame| frame.symbols())
        .zip(1usize..)
        .map(|(sym, n)| Frame {
          n,
          name: sym.name().map(|n| n.to_string()),
          lineno: sym.lineno(),
          filename: sym.filename().map(|x| x.into()),
        })
        .collect();

      let mut filtered_frames: Vec<_> = frames.iter().collect();
      apply_frames_filters(&mut filtered_frames);

      if !filtered_frames.is_empty() {
        writeln!(f, "Backtrace:")?;
        for frame in &filtered_frames {
          let file = frame.filename.as_ref().map(|path| path.display());
          let file: &dyn fmt::Display = if let Some(ref filename) = file {
            filename
          } else {
            &"<unknown source file>"
          };

          writeln!(
            f,
            "  [{}] {} ({}:{})",
            frame.n,
            frame.name.as_deref().unwrap_or("<unknown>"),
            file,
            frame.lineno.unwrap_or(0)
          )?;
        }
      }
    }

    Ok(())
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "code: {} | source: {} | contexts: {}",
      self.code,
      self.source_summary(),
      self.context_summary()
    )
  }
}

impl StdError for Error {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    self
      .source
      .as_ref()
      .map(|e| e.as_ref() as &(dyn StdError + 'static))
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Self::from_std(err)
  }
}

impl From<anyhow::Error> for Error {
  fn from(err: anyhow::Error) -> Self {
    let contexts: Vec<String> = err.chain().skip(1).map(|e| e.to_string()).collect();

    Self {
      code: BODHIERR_SYS,
      source: None,
      contexts: Some(contexts),
      backtrace: None,
    }
    .capture_backtrace()
  }
}
