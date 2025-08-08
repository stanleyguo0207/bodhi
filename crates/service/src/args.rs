use clap::Parser;

fn validate_dir(s: &str) -> Result<std::path::PathBuf, String> {
  let path = std::path::PathBuf::from(s);

  if !path.exists() {
    return Err(format!("路径 '{}' 不存在", s));
  }

  if !path.is_dir() {
    return Err(format!("'{}' 不是目录", s));
  }

  Ok(path)
}

/// 命令行参数节解析器
#[derive(Parser, Debug, PartialEq)]
#[command(version, about, long_about = None)]
pub struct Args {
  /// 设置配置目录路径
  #[arg(
    long = "cfgd",
    value_name = "DIRECTORY",
    default_value = "./config",
    value_hint = clap::ValueHint::DirPath,
    value_parser = validate_dir,
    help = "指定配置目录的路径"
  )]
  pub config_dir: std::path::PathBuf,
}
