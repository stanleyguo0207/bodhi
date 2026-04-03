//! 配置加载模块

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use bodhi_error::prelude::*;
use toml::Value;

use crate::errcode::configerr::*;
use crate::merge::merge_all;

pub fn ensure_config_dir(config_dir: &Path) -> Result<()> {
  if config_dir.is_dir() {
    Ok(())
  } else {
    Err(
      Error::new(CONFIGERR_CONFIGDIRNOTFOUND)
        .wrap_context_with(|| format!("config_dir={} not found", config_dir.display())),
    )
  }
}

pub fn discover_services(config_dir: &Path) -> Result<Vec<String>> {
  let service_dir = config_dir.join("template").join("service");
  list_toml_stems(&service_dir, CONFIGERR_TEMPLATEDIRNOTFOUND)
}

pub fn discover_profiles(config_dir: &Path) -> Result<Vec<String>> {
  let profile_dir = config_dir.join("profile");
  list_toml_stems(&profile_dir, CONFIGERR_PROFILEDIRNOTFOUND)
}

pub fn load_infra_configs(config_dir: &Path) -> Result<Value> {
  let infra_dir = config_dir.join("template").join("infra");
  let paths = list_toml_files(&infra_dir, CONFIGERR_TEMPLATEDIRNOTFOUND)?;
  let mut values = Vec::with_capacity(paths.len());
  for path in paths {
    values.push(load_toml_file(&path)?);
  }
  Ok(merge_all(values))
}

pub fn load_service_template(config_dir: &Path, service: &str) -> Result<Value> {
  let path = config_dir
    .join("template")
    .join("service")
    .join(format!("{service}.toml"));

  if !path.is_file() {
    return Err(
      Error::new(CONFIGERR_SERVICENOTFOUND)
        .wrap_context_with(|| format!("service={service} path={} not found", path.display())),
    );
  }

  load_toml_file(&path)
}

pub fn load_service_templates(config_dir: &Path) -> Result<BTreeMap<String, Value>> {
  let services = discover_services(config_dir)?;
  let mut templates = BTreeMap::new();

  for service in services {
    let template = load_service_template(config_dir, &service)?;
    templates.insert(service, template);
  }

  Ok(templates)
}

pub fn load_profile(config_dir: &Path, profile: &str) -> Result<Value> {
  let path = config_dir.join("profile").join(format!("{profile}.toml"));

  if !path.is_file() {
    return Err(
      Error::new(CONFIGERR_PROFILENOTFOUND)
        .wrap_context_with(|| format!("profile={profile} path={} not found", path.display())),
    );
  }

  load_toml_file(&path)
}

pub fn load_toml_file(path: &Path) -> Result<Value> {
  let content = fs::read_to_string(path)
    .map_err(Error::from_std)
    .wrap_context("read config file failed")
    .wrap_context_with(|| format!("path={}", path.display()))?;

  toml::from_str::<Value>(&content)
    .map_err(Error::from_std)
    .wrap_context("parse toml file failed")
    .wrap_context_with(|| format!("path={}", path.display()))
}

fn list_toml_files(dir: &Path, missing_code: i32) -> Result<Vec<PathBuf>> {
  if !dir.is_dir() {
    return Err(
      Error::new(missing_code).wrap_context_with(|| format!("dir={} not found", dir.display())),
    );
  }

  let mut paths = Vec::new();
  for entry in fs::read_dir(dir)
    .map_err(Error::from_std)
    .wrap_context("read config directory failed")
    .wrap_context_with(|| format!("dir={}", dir.display()))?
  {
    let entry = entry
      .map_err(Error::from_std)
      .wrap_context("read directory entry failed")
      .wrap_context_with(|| format!("dir={}", dir.display()))?;
    let path = entry.path();
    if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
      paths.push(path);
    }
  }

  paths.sort();
  Ok(paths)
}

fn list_toml_stems(dir: &Path, missing_code: i32) -> Result<Vec<String>> {
  let mut names = Vec::new();
  for path in list_toml_files(dir, missing_code)? {
    let stem = path
      .file_stem()
      .and_then(|value| value.to_str())
      .ok_or_else(|| Error::new(CONFIGERR_FILELOADFAILED))
      .wrap_context("get file stem failed")
      .wrap_context_with(|| format!("path={}", path.display()))?;
    names.push(stem.to_string());
  }

  Ok(names)
}
