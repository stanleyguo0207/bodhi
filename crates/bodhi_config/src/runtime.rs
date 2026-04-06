use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use bodhi_error::prelude::*;
use serde::de::DeserializeOwned;

use crate::engine::{ConfigEngine, ResolvedLayers};

/// 单次装载后的配置快照
#[derive(Clone)]
pub struct ConfigSnapshot<I, S> {
  version: u64,
  layers: Arc<ResolvedLayers>,
  infra: Arc<I>,
  service: Arc<S>,
}

impl<I, S> ConfigSnapshot<I, S> {
  pub fn version(&self) -> u64 {
    self.version
  }

  pub fn layers(&self) -> &ResolvedLayers {
    self.layers.as_ref()
  }

  pub fn layers_arc(&self) -> Arc<ResolvedLayers> {
    Arc::clone(&self.layers)
  }

  pub fn infra(&self) -> &I {
    self.infra.as_ref()
  }

  pub fn infra_arc(&self) -> Arc<I> {
    Arc::clone(&self.infra)
  }

  pub fn service(&self) -> &S {
    self.service.as_ref()
  }

  pub fn service_arc(&self) -> Arc<S> {
    Arc::clone(&self.service)
  }
}

/// 线程安全的配置运行时存储
pub struct ConfigStore<I, S> {
  engine: ConfigEngine,
  profile: String,
  service: String,
  next_version: AtomicU64,
  state: RwLock<Arc<ConfigSnapshot<I, S>>>,
}

impl<I, S> ConfigStore<I, S>
where
  I: DeserializeOwned + Send + Sync + 'static,
  S: DeserializeOwned + Send + Sync + 'static,
{
  pub fn load(profile: &str, service: &str) -> Result<Self> {
    Self::load_from("config", profile, service)
  }

  pub fn load_from(config_dir: impl AsRef<Path>, profile: &str, service: &str) -> Result<Self> {
    let engine = ConfigEngine::find(config_dir)?;
    Self::from_engine(engine, profile, service)
  }

  pub fn from_engine(engine: ConfigEngine, profile: &str, service: &str) -> Result<Self> {
    let initial_version = 1;
    let snapshot = Arc::new(Self::load_snapshot(
      &engine,
      profile,
      service,
      initial_version,
    )?);

    Ok(Self {
      engine,
      profile: profile.to_string(),
      service: service.to_string(),
      next_version: AtomicU64::new(initial_version + 1),
      state: RwLock::new(snapshot),
    })
  }

  pub fn profile(&self) -> &str {
    &self.profile
  }

  pub fn service(&self) -> &str {
    &self.service
  }

  pub fn config_dir(&self) -> &Path {
    self.engine.config_dir()
  }

  pub fn snapshot(&self) -> Arc<ConfigSnapshot<I, S>> {
    Arc::clone(&self.read_state())
  }

  pub fn reload(&self) -> Result<Arc<ConfigSnapshot<I, S>>> {
    let version = self.next_version.fetch_add(1, Ordering::AcqRel);
    let snapshot = Arc::new(Self::load_snapshot(
      &self.engine,
      &self.profile,
      &self.service,
      version,
    )?);

    *self.write_state() = Arc::clone(&snapshot);
    Ok(snapshot)
  }

  pub fn current_version(&self) -> u64 {
    self.snapshot().version()
  }

  fn load_snapshot(
    engine: &ConfigEngine,
    profile: &str,
    service: &str,
    version: u64,
  ) -> Result<ConfigSnapshot<I, S>> {
    let layers = engine.resolve_layers(profile, service)?;
    let infra = layers.extract_infra(".")?;
    let service_cfg = layers.extract_service(".")?;

    Ok(ConfigSnapshot {
      version,
      layers: Arc::new(layers),
      infra: Arc::new(infra),
      service: Arc::new(service_cfg),
    })
  }

  fn read_state(&self) -> std::sync::RwLockReadGuard<'_, Arc<ConfigSnapshot<I, S>>> {
    self.state.read().unwrap_or_else(|err| err.into_inner())
  }

  fn write_state(&self) -> std::sync::RwLockWriteGuard<'_, Arc<ConfigSnapshot<I, S>>> {
    self.state.write().unwrap_or_else(|err| err.into_inner())
  }
}
