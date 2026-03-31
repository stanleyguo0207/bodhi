//! 错误码注册模块

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use super::BODHIERR_SYS;
use crate::error::{Error, Result};

/// 全局错误码注册表
static ERROR_CODE_REGISTRY: LazyLock<RwLock<HashMap<i32, &'static str>>> =
  LazyLock::new(|| RwLock::new(HashMap::new()));

/// 注册错误码
pub fn register_error_code(code: i32, meta: &'static str) -> Result<()> {
  let mut registry = ERROR_CODE_REGISTRY.write().map_err(|e| {
    Error::new(BODHIERR_SYS).wrap_context_with(|| format!("Failed to acquire registry lock: {}", e))
  })?;

  if let Some(existing) = registry.get(&code) {
    return Err(Error::new(BODHIERR_SYS).wrap_context_with(|| {
      format!(
        "Error code {} already registered for '{}', cannot register for '{}'",
        code, existing, meta
      )
    }));
  }

  registry.insert(code, meta);
  Ok(())
}
