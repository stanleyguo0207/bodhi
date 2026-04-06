//! 配置引擎模块

use std::env;
use std::path::{Path, PathBuf};

use bodhi_error::prelude::*;
use serde::de::DeserializeOwned;
use toml::Value;

use crate::codegen::{
  RustCodegenOptions, RustCodegenResult, render_layered_rust_types,
  render_layered_rust_types_report, render_rust_types, render_rust_types_report, write_rust_types,
};
use crate::errcode::configerr::*;
use crate::loader::{discover_profiles, discover_services, ensure_config_dir, find_config_dir};
use crate::merge::deep_merge;
use crate::output::{OutputFormat, serialize_value, write_product};

/// 配置引擎
#[derive(Debug)]
pub struct ConfigEngine {
  config_dir: PathBuf,
}

impl ConfigEngine {
  /// 创建配置引擎
  pub fn new(config_dir: impl AsRef<Path>) -> Result<Self> {
    let config_dir = config_dir.as_ref().to_path_buf();
    ensure_config_dir(&config_dir)?;
    Ok(Self { config_dir })
  }

  /// 从当前工作目录向上查找配置目录并创建配置引擎
  pub fn find(config_dir: impl AsRef<Path>) -> Result<Self> {
    let current_dir = env::current_dir()
      .map_err(Error::from_std)
      .wrap_context("get current directory failed")?;

    Self::find_from(current_dir, config_dir)
  }

  /// 从指定起始目录向上查找配置目录并创建配置引擎
  pub fn find_from(start_dir: impl AsRef<Path>, config_dir: impl AsRef<Path>) -> Result<Self> {
    let config_dir = find_config_dir(start_dir.as_ref(), config_dir.as_ref())?;
    Ok(Self { config_dir })
  }

  /// 获取配置根目录
  pub fn config_dir(&self) -> &Path {
    &self.config_dir
  }

  /// 列出所有服务
  pub fn services(&self) -> Result<Vec<String>> {
    discover_services(&self.config_dir)
  }

  /// 列出所有 profile
  pub fn profiles(&self) -> Result<Vec<String>> {
    discover_profiles(&self.config_dir)
  }

  /// 解析指定 profile 和 service 的最终配置
  pub fn resolve(&self, profile: &str, service: &str) -> Result<ResolvedConfig> {
    crate::resolve::resolve(&self.config_dir, profile, service)
  }

  /// 解析指定 profile 和 service 的分层配置
  pub fn resolve_layers(&self, profile: &str, service: &str) -> Result<ResolvedLayers> {
    crate::resolve::resolve_layers(&self.config_dir, profile, service)
  }

  /// 解析指定 service 的配置结构
  pub fn resolve_service_schema(&self, service: &str) -> Result<ResolvedConfig> {
    crate::resolve::resolve_service_schema(&self.config_dir, service)
  }

  /// 解析指定 service 的分层配置结构
  pub fn resolve_service_schema_layers(&self, service: &str) -> Result<ResolvedLayers> {
    crate::resolve::resolve_service_schema_layers(&self.config_dir, service)
  }

  /// 生成指定 profile 下全部服务的产物
  pub fn generate(&self, profile: &str, formats: &[OutputFormat]) -> Result<()> {
    let services = self.services()?;
    for service in services {
      self.generate_service(profile, &service, formats)?;
    }
    Ok(())
  }

  /// 生成指定服务的产物
  pub fn generate_service(
    &self,
    profile: &str,
    service: &str,
    formats: &[OutputFormat],
  ) -> Result<()> {
    let resolved = self.resolve(profile, service)?;
    let formats = if formats.is_empty() {
      OutputFormat::all().to_vec()
    } else {
      formats.to_vec()
    };

    for format in formats {
      write_product(&self.config_dir, profile, service, resolved.value(), format)?;
    }

    Ok(())
  }

  /// 渲染指定服务的 Rust 配置结构定义
  pub fn render_rust_types(&self, profile: &str, service: &str) -> Result<String> {
    self.render_rust_types_with(profile, service, &RustCodegenOptions::default())
  }

  /// 按 service 配置结构渲染 Rust 配置结构定义
  pub fn render_service_rust_types(&self, service: &str) -> Result<String> {
    self.render_service_rust_types_with(service, &RustCodegenOptions::default())
  }

  /// 按指定选项渲染 Rust 配置结构定义
  pub fn render_rust_types_with(
    &self,
    profile: &str,
    service: &str,
    options: &RustCodegenOptions,
  ) -> Result<String> {
    let resolved = self.resolve_layers(profile, service)?;
    render_layered_rust_types(
      resolved.infra(),
      resolved.service(),
      resolved.merged(),
      options,
    )
  }

  /// 按 service 配置结构和指定选项渲染 Rust 配置结构定义
  pub fn render_service_rust_types_with(
    &self,
    service: &str,
    options: &RustCodegenOptions,
  ) -> Result<String> {
    let resolved = self.resolve_service_schema_layers(service)?;
    render_layered_rust_types(
      resolved.infra(),
      resolved.service(),
      resolved.merged(),
      options,
    )
  }

  /// 渲染指定服务的 Rust 配置结构定义和规则命中报告
  pub fn render_rust_types_report(
    &self,
    profile: &str,
    service: &str,
  ) -> Result<RustCodegenResult> {
    self.render_rust_types_report_with(profile, service, &RustCodegenOptions::default())
  }

  /// 按 service 配置结构渲染 Rust 配置结构定义和规则命中报告
  pub fn render_service_rust_types_report(&self, service: &str) -> Result<RustCodegenResult> {
    self.render_service_rust_types_report_with(service, &RustCodegenOptions::default())
  }

  /// 按指定选项渲染 Rust 配置结构定义和规则命中报告
  pub fn render_rust_types_report_with(
    &self,
    profile: &str,
    service: &str,
    options: &RustCodegenOptions,
  ) -> Result<RustCodegenResult> {
    let resolved = self.resolve_layers(profile, service)?;
    render_layered_rust_types_report(
      resolved.infra(),
      resolved.service(),
      resolved.merged(),
      options,
    )
  }

  /// 按 service 配置结构和指定选项渲染 Rust 配置结构定义和规则命中报告
  pub fn render_service_rust_types_report_with(
    &self,
    service: &str,
    options: &RustCodegenOptions,
  ) -> Result<RustCodegenResult> {
    let resolved = self.resolve_service_schema_layers(service)?;
    render_layered_rust_types_report(
      resolved.infra(),
      resolved.service(),
      resolved.merged(),
      options,
    )
  }

  /// 生成指定服务的 Rust 配置结构文件
  pub fn generate_rust_types(
    &self,
    profile: &str,
    service: &str,
    output_path: impl AsRef<Path>,
  ) -> Result<()> {
    self.generate_rust_types_with(
      profile,
      service,
      output_path,
      &RustCodegenOptions::default(),
    )
  }

  /// 按 service 配置结构生成 Rust 配置结构文件
  pub fn generate_service_rust_types(
    &self,
    service: &str,
    output_path: impl AsRef<Path>,
  ) -> Result<()> {
    self.generate_service_rust_types_with(service, output_path, &RustCodegenOptions::default())
  }

  /// 按指定选项生成 Rust 配置结构文件
  pub fn generate_rust_types_with(
    &self,
    profile: &str,
    service: &str,
    output_path: impl AsRef<Path>,
    options: &RustCodegenOptions,
  ) -> Result<()> {
    let content = self.render_rust_types_with(profile, service, options)?;
    write_rust_types(output_path.as_ref(), &content)
  }

  /// 按 service 配置结构和指定选项生成 Rust 配置结构文件
  pub fn generate_service_rust_types_with(
    &self,
    service: &str,
    output_path: impl AsRef<Path>,
    options: &RustCodegenOptions,
  ) -> Result<()> {
    let content = self.render_service_rust_types_with(service, options)?;
    write_rust_types(output_path.as_ref(), &content)
  }

  /// 生成指定服务的 Rust 配置结构文件并返回规则命中报告
  pub fn generate_rust_types_report_with(
    &self,
    profile: &str,
    service: &str,
    output_path: impl AsRef<Path>,
    options: &RustCodegenOptions,
  ) -> Result<RustCodegenResult> {
    let result = self.render_rust_types_report_with(profile, service, options)?;
    write_rust_types(output_path.as_ref(), &result.content)?;
    Ok(result)
  }

  /// 按 service 配置结构生成 Rust 配置结构文件并返回规则命中报告
  pub fn generate_service_rust_types_report_with(
    &self,
    service: &str,
    output_path: impl AsRef<Path>,
    options: &RustCodegenOptions,
  ) -> Result<RustCodegenResult> {
    let result = self.render_service_rust_types_report_with(service, options)?;
    write_rust_types(output_path.as_ref(), &result.content)?;
    Ok(result)
  }

  /// 批量生成指定 profile 下全部服务的 Rust 配置结构文件
  pub fn generate_all_rust_types(&self, profile: &str, output_dir: impl AsRef<Path>) -> Result<()> {
    self.generate_all_rust_types_with(profile, output_dir, &RustCodegenOptions::default())
  }

  /// 按服务名前缀批量生成 Rust 配置结构文件
  pub fn generate_prefixed_rust_types(
    &self,
    profile: &str,
    service_prefix: &str,
    output_dir: impl AsRef<Path>,
  ) -> Result<()> {
    self.generate_prefixed_rust_types_with(
      profile,
      service_prefix,
      output_dir,
      &RustCodegenOptions::default(),
    )
  }

  /// 按指定选项批量生成指定 profile 下全部服务的 Rust 配置结构文件
  pub fn generate_all_rust_types_with(
    &self,
    profile: &str,
    output_dir: impl AsRef<Path>,
    options: &RustCodegenOptions,
  ) -> Result<()> {
    let output_dir = output_dir.as_ref();
    let services = self.services()?;

    self.generate_rust_types_for_services(profile, &services, output_dir, options)
  }

  /// 按指定选项和服务名前缀批量生成 Rust 配置结构文件
  pub fn generate_prefixed_rust_types_with(
    &self,
    profile: &str,
    service_prefix: &str,
    output_dir: impl AsRef<Path>,
    options: &RustCodegenOptions,
  ) -> Result<()> {
    let output_dir = output_dir.as_ref();
    let services = self.services_with_prefix(service_prefix)?;

    self.generate_rust_types_for_services(profile, &services, output_dir, options)
  }

  /// 获取匹配指定前缀的服务
  pub fn services_with_prefix(&self, service_prefix: &str) -> Result<Vec<String>> {
    let matched: Vec<_> = self
      .services()?
      .into_iter()
      .filter(|service| service.starts_with(service_prefix))
      .collect();

    if matched.is_empty() {
      return Err(
        Error::new(CONFIGERR_SERVICENOTFOUND)
          .wrap_context("no services matched service prefix")
          .wrap_context_with(|| format!("service_prefix={service_prefix}")),
      );
    }

    Ok(matched)
  }

  fn generate_rust_types_for_services(
    &self,
    profile: &str,
    services: &[String],
    output_dir: &Path,
    options: &RustCodegenOptions,
  ) -> Result<()> {
    for service in services {
      let output_path = output_dir.join(format!("{}_config.rs", service));
      self.generate_rust_types_with(profile, service, output_path, options)?;
    }

    Ok(())
  }

  /// 获取默认 Rust 结构输出目录
  pub fn default_rust_output_dir(&self, profile: &str) -> PathBuf {
    self.config_dir.join("product").join(profile).join("rust")
  }

  /// 获取 workspace 级 Rust 结构输出目录
  pub fn default_target_rust_output_dir(&self) -> PathBuf {
    self.workspace_root().join("target").join("bodhi_config")
  }

  /// 获取默认 Rust 结构输出路径
  pub fn default_rust_output_path(&self, profile: &str, service: &str) -> PathBuf {
    self
      .default_rust_output_dir(profile)
      .join(format!("{}_config.rs", service))
  }

  /// 获取 workspace 级服务 Rust 结构输出路径
  pub fn default_target_rust_output_path(&self, service: &str) -> PathBuf {
    self
      .default_target_rust_output_dir()
      .join(service)
      .join("config.rs")
  }

  fn workspace_root(&self) -> &Path {
    self
      .config_dir
      .parent()
      .unwrap_or(self.config_dir.as_path())
  }
}

/// 分层解析后的配置
#[derive(Clone, Debug)]
pub struct ResolvedLayers {
  infra: Value,
  service: Value,
  merged: Value,
}

impl ResolvedLayers {
  pub(crate) fn new(infra: Value, service: Value) -> Result<Self> {
    ensure_table(&infra, "infra config must be a table")?;
    ensure_table(&service, "service config must be a table")?;

    let mut merged = infra.clone();
    deep_merge(&mut merged, &service);
    ensure_table(&merged, "merged config must be a table")?;

    Ok(Self {
      infra,
      service,
      merged,
    })
  }

  /// 获取 infra 层配置原始值
  pub fn infra(&self) -> &Value {
    &self.infra
  }

  /// 获取 service 层配置原始值
  pub fn service(&self) -> &Value {
    &self.service
  }

  /// 获取最终合并后的配置原始值
  pub fn merged(&self) -> &Value {
    &self.merged
  }

  /// 提取指定路径的 infra 层类型化配置
  pub fn extract_infra<T>(&self, path: &str) -> Result<T>
  where
    T: DeserializeOwned,
  {
    extract_typed_value(&self.infra, path, "infra")
  }

  /// 提取指定路径的 service 层类型化配置
  pub fn extract_service<T>(&self, path: &str) -> Result<T>
  where
    T: DeserializeOwned,
  {
    extract_typed_value(&self.service, path, "service")
  }

  /// 提取指定路径的最终合并类型化配置
  pub fn extract_merged<T>(&self, path: &str) -> Result<T>
  where
    T: DeserializeOwned,
  {
    extract_typed_value(&self.merged, path, "merged")
  }

  /// 消费并返回分层配置
  pub fn into_parts(self) -> (Value, Value, Value) {
    (self.infra, self.service, self.merged)
  }

  /// 转换为现有的最终配置对象
  pub fn into_resolved_config(self) -> ResolvedConfig {
    ResolvedConfig::new(self.merged)
  }
}

/// 最终解析后的配置
#[derive(Debug)]
pub struct ResolvedConfig {
  value: Value,
}

impl ResolvedConfig {
  pub(crate) fn new(value: Value) -> Self {
    Self { value }
  }

  /// 获取配置原始值
  pub fn value(&self) -> &Value {
    &self.value
  }

  /// 消费并返回原始值
  pub fn into_value(self) -> Value {
    self.value
  }

  /// 提取指定路径的类型化配置
  pub fn extract<T>(&self, path: &str) -> Result<T>
  where
    T: DeserializeOwned,
  {
    extract_typed_value(&self.value, path, "merged")
  }

  /// 序列化为指定格式
  pub fn to_format(&self, format: OutputFormat) -> Result<String> {
    serialize_value(&self.value, format)
  }

  /// 渲染当前配置对应的 Rust 结构定义
  pub fn to_rust_types(&self) -> Result<String> {
    render_rust_types(&self.value, &RustCodegenOptions::default())
  }

  /// 渲染当前配置对应的 Rust 结构定义和规则命中报告
  pub fn to_rust_types_report(&self) -> Result<RustCodegenResult> {
    render_rust_types_report(&self.value, &RustCodegenOptions::default())
  }
}

fn ensure_table(value: &Value, reason: &str) -> Result<()> {
  if matches!(value, Value::Table(_)) {
    Ok(())
  } else {
    Err(Error::new(CONFIGERR_INVALIDSTRUCTURE).wrap_context(reason))
  }
}

fn extract_typed_value<T>(value: &Value, path: &str, scope: &str) -> Result<T>
where
  T: DeserializeOwned,
{
  let target = if path.is_empty() || path == "." {
    value.clone()
  } else {
    get_path(value, path)?.clone()
  };

  target
    .try_into()
    .map_err(Error::from_std)
    .wrap_context("extract typed config failed")
    .wrap_context_with(|| format!("scope={scope} path={path}"))
}

fn get_path<'a>(value: &'a Value, path: &str) -> Result<&'a Value> {
  let mut current = value;
  for segment in path.split('.') {
    if segment.is_empty() {
      return Err(Error::new(CONFIGERR_INVALIDPATH).wrap_context_with(|| format!("path={path}")));
    }

    let table = current
      .as_table()
      .ok_or_else(|| Error::new(CONFIGERR_INVALIDPATH))
      .wrap_context("path segment target is not a table")
      .wrap_context_with(|| format!("path={path} segment={segment}"))?;

    current = table
      .get(segment)
      .ok_or_else(|| Error::new(CONFIGERR_EXTRACTFAILED))
      .wrap_context("path segment not found")
      .wrap_context_with(|| format!("path={path} segment={segment}"))?;
  }

  Ok(current)
}
