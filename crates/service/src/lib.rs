use args::Args;
use clap::Parser;

mod args;

#[cfg(test)]
mod args_test;

pub fn serve() {
  let args = Args::parse();
  println!("配置目录: {}", args.config_dir.display())
}
