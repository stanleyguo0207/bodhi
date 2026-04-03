//! 配置引擎模块

use std::path::{Path, PathBuf};

use bodhi_error::prelude::*;
use serde::de::DeserializeOwned;
use toml::Value;

use crate::codegen::{
  RustCodegenOptions, RustCodegenResult, render_rust_types, render_rust_types_report,
  write_rust_types,
};
use crate::errcode::configerr::*;
use crate::loader::{discover_profiles, discover_services, ensure_config_dir};
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

  /// 按指定选项渲染 Rust 配置结构定义
  pub fn render_rust_types_with(
    &self,
    profile: &str,
    service: &str,
    options: &RustCodegenOptions,
  ) -> Result<String> {
    Ok(
      self
        .render_rust_types_report_with(profile, service, options)?
        .content,
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

  /// 按指定选项渲染 Rust 配置结构定义和规则命中报告
  pub fn render_rust_types_report_with(
    &self,
    profile: &str,
    service: &str,
    options: &RustCodegenOptions,
  ) -> Result<RustCodegenResult> {
    let resolved = self.resolve(profile, service)?;
    render_rust_types_report(resolved.value(), options)
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

  /// 获取默认 Rust 结构输出路径
  pub fn default_rust_output_path(&self, profile: &str, service: &str) -> PathBuf {
    self
      .default_rust_output_dir(profile)
      .join(format!("{}_config.rs", service))
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
    let target = if path.is_empty() || path == "." {
      self.value.clone()
    } else {
      get_path(&self.value, path)?.clone()
    };

    target
      .try_into()
      .map_err(Error::from_std)
      .wrap_context("extract typed config failed")
      .wrap_context_with(|| format!("path={path}"))
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
