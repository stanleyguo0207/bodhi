use std::path::PathBuf;

/// 运行时参数（对外暴露）
#[derive(Debug, Clone)]
pub struct Args {
  /// 表示应用读取配置的目录
  pub config_dir: PathBuf,
}
