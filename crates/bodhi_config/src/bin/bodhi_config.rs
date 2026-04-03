use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use bodhi_config::codegen::write_rust_types;
use bodhi_config::prelude::*;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "bodhi_config")]
#[command(about = "Bodhi 配置产物生成工具")]
struct Cli {
  #[arg(long, default_value = "config")]
  config_dir: PathBuf,
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  /// 列出可用的 profile 和 service
  List,
  /// 为整个项目生成配置产物和 Rust 配置结构
  GenProject {
    #[arg(long)]
    rust_output: Option<PathBuf>,
    #[arg(long)]
    type_rules: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "text")]
    report_format: RuleReportFormat,
    #[arg(long, requires = "type_rules")]
    report_output: Option<PathBuf>,
    #[arg(long, default_value = "Config")]
    root_struct: String,
  },
  /// 生成配置产物
  Gen {
    #[arg(long)]
    profile: String,
    #[arg(long)]
    service: Option<String>,
    #[arg(long = "format")]
    formats: Vec<String>,
  },
  /// 展示最终合并后的配置
  Show {
    #[arg(long)]
    profile: String,
    #[arg(long)]
    service: String,
    #[arg(long, default_value = "toml")]
    format: String,
  },
  /// 生成 Rust 配置结构定义文件
  GenRust {
    #[arg(long)]
    profile: String,
    #[arg(long, conflicts_with = "service_prefix")]
    service: Option<String>,
    #[arg(long, conflicts_with = "service")]
    service_prefix: Option<String>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long)]
    type_rules: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "text")]
    report_format: RuleReportFormat,
    #[arg(long, requires = "type_rules")]
    report_output: Option<PathBuf>,
    #[arg(long, default_value = "Config")]
    root_struct: String,
  },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum RuleReportFormat {
  Text,
  Json,
}

#[derive(Clone, Debug, Serialize)]
struct GeneratedServiceReport {
  service: String,
  output: String,
  matched_rules: Vec<TypeOverrideHit>,
  unused_rules: Vec<TypeOverrideRule>,
}

#[derive(Debug, Serialize)]
struct GenRustRuleReport {
  profile: String,
  generated: Vec<GeneratedServiceReport>,
  unused_rules: Vec<TypeOverrideRule>,
}

fn main() -> Result<()> {
  let cli = Cli::parse();
  let engine = ConfigEngine::new(&cli.config_dir)?;

  match cli.command {
    Command::List => {
      println!("profiles:");
      for profile in engine.profiles()? {
        println!("  {profile}");
      }

      println!("services:");
      for service in engine.services()? {
        println!("  {service}");
      }
    }
    Command::GenProject {
      rust_output,
      type_rules,
      report_format,
      report_output,
      root_struct,
    } => {
      let runtime_formats = [OutputFormat::Toml, OutputFormat::Json, OutputFormat::Yaml];
      for profile in engine.profiles()? {
        engine.generate(&profile, &runtime_formats)?;
        println!("generated runtime products for profile {profile}");
      }

      let type_overrides = if let Some(type_rules) = type_rules.as_ref() {
        TypeOverrideRules::from_file(type_rules)?
      } else {
        TypeOverrideRules::default()
      };
      let show_rule_report = type_rules.is_some();
      let options = RustCodegenOptions {
        root_struct_name: root_struct,
        type_overrides: type_overrides.clone(),
      };
      let rust_output = rust_output.unwrap_or_else(|| engine.default_ide_rust_output_dir());
      let mut generated = Vec::new();

      for service in engine.services()? {
        let output_path = rust_output.join(&service).join("config.rs");
        generated.push(generate_service_schema_report(
          &engine,
          &service,
          output_path,
          &options,
        )?);
      }

      if show_rule_report {
        let mut matched_rules = Vec::new();
        for service_report in &generated {
          matched_rules.extend(service_report.matched_rules.iter().cloned());
        }

        let unused_rules = type_overrides.find_unused_rules(&matched_rules);
        let report = match report_format {
          RuleReportFormat::Text => render_rule_report_text(&generated, &unused_rules),
          RuleReportFormat::Json => {
            render_rule_report_json("workspace", &generated, &unused_rules)?
          }
        };
        emit_rule_report(&report, report_output.as_deref())?;
      } else {
        for service_report in &generated {
          println!("generated {}", service_report.output);
        }
      }
    }
    Command::Gen {
      profile,
      service,
      formats,
    } => {
      let formats = if formats.is_empty() {
        OutputFormat::all().to_vec()
      } else {
        let mut parsed = Vec::with_capacity(formats.len());
        for format in formats {
          parsed.push(format.parse()?);
        }
        parsed
      };

      if let Some(service) = service {
        engine.generate_service(&profile, &service, &formats)?;
      } else {
        engine.generate(&profile, &formats)?;
      }
    }
    Command::Show {
      profile,
      service,
      format,
    } => {
      let format: OutputFormat = format.parse()?;
      let resolved = engine.resolve(&profile, &service)?;
      let content = resolved.to_format(format)?;
      print!("{content}");
      if !content.ends_with('\n') {
        println!();
      }
    }
    Command::GenRust {
      profile,
      service,
      service_prefix,
      output,
      type_rules,
      report_format,
      report_output,
      root_struct,
    } => {
      let type_overrides = if let Some(type_rules) = type_rules.as_ref() {
        TypeOverrideRules::from_file(type_rules)?
      } else {
        TypeOverrideRules::default()
      };

      let options = RustCodegenOptions {
        root_struct_name: root_struct,
        type_overrides: type_overrides.clone(),
      };
      let show_rule_report = type_rules.is_some();
      let mut generated = Vec::new();

      if let Some(service) = service {
        let output = output.unwrap_or_else(|| engine.default_rust_output_path(&profile, &service));
        generated.push(generate_service_report(
          &engine, &profile, &service, output, &options,
        )?);
      } else if let Some(service_prefix) = service_prefix {
        let output_dir = output.unwrap_or_else(|| engine.default_rust_output_dir(&profile));
        ensure_batch_output_dir(&output_dir)?;

        let services = engine.services_with_prefix(&service_prefix)?;
        for service in services {
          let output_path = output_dir.join(format!("{}_config.rs", service));
          generated.push(generate_service_report(
            &engine,
            &profile,
            &service,
            output_path,
            &options,
          )?);
        }
      } else {
        let output_dir = output.unwrap_or_else(|| engine.default_rust_output_dir(&profile));
        ensure_batch_output_dir(&output_dir)?;

        for service in engine.services()? {
          let output_path = output_dir.join(format!("{}_config.rs", service));
          generated.push(generate_service_report(
            &engine,
            &profile,
            &service,
            output_path,
            &options,
          )?);
        }
      }

      if show_rule_report {
        let mut matched_rules = Vec::new();
        for service_report in &generated {
          matched_rules.extend(service_report.matched_rules.iter().cloned());
        }

        let unused_rules = type_overrides.find_unused_rules(&matched_rules);
        let report = match report_format {
          RuleReportFormat::Text => render_rule_report_text(&generated, &unused_rules),
          RuleReportFormat::Json => render_rule_report_json(&profile, &generated, &unused_rules)?,
        };
        emit_rule_report(&report, report_output.as_deref())?;
      } else {
        for service_report in &generated {
          println!("generated {}", service_report.output);
        }
      }
    }
  }

  Ok(())
}

fn ensure_batch_output_dir(output_dir: &Path) -> Result<()> {
  if output_dir.extension().is_some() {
    return Err(
      Error::new(CONFIGERR_INVALIDPATH)
        .wrap_context("batch gen-rust output must be a directory")
        .wrap_context_with(|| format!("path={}", output_dir.display())),
    );
  }

  Ok(())
}

fn generate_service_report(
  engine: &ConfigEngine,
  profile: &str,
  service: &str,
  output: PathBuf,
  options: &RustCodegenOptions,
) -> Result<GeneratedServiceReport> {
  let report = engine.generate_rust_types_report_with(profile, service, &output, options)?;
  Ok(GeneratedServiceReport {
    service: service.to_string(),
    output: output.display().to_string(),
    matched_rules: report.matched_rules,
    unused_rules: report.unused_rules,
  })
}

fn generate_service_schema_report(
  engine: &ConfigEngine,
  service: &str,
  output: PathBuf,
  options: &RustCodegenOptions,
) -> Result<GeneratedServiceReport> {
  let report = engine.render_service_rust_types_report_with(service, options)?;
  write_rust_types(&output, &report.content)?;

  Ok(GeneratedServiceReport {
    service: service.to_string(),
    output: output.display().to_string(),
    matched_rules: report.matched_rules,
    unused_rules: report.unused_rules,
  })
}

fn render_rule_report_text(
  generated: &[GeneratedServiceReport],
  unused_rules: &[TypeOverrideRule],
) -> String {
  let mut output = String::new();

  for service_report in generated {
    writeln!(&mut output, "generated {}", service_report.output).expect("write string");
    append_matched_rules(
      &mut output,
      &service_report.service,
      &service_report.matched_rules,
    );
  }

  append_unused_rules(&mut output, unused_rules);
  output
}

fn render_rule_report_json(
  profile: &str,
  generated: &[GeneratedServiceReport],
  unused_rules: &[TypeOverrideRule],
) -> Result<String> {
  let report = GenRustRuleReport {
    profile: profile.to_string(),
    generated: generated.to_vec(),
    unused_rules: unused_rules.to_vec(),
  };

  serde_json::to_string_pretty(&report)
    .map_err(Error::from_std)
    .wrap_context("serialize gen-rust rule report failed")
}

fn emit_rule_report(report: &str, report_output: Option<&Path>) -> Result<()> {
  if let Some(report_output) = report_output {
    if let Some(parent) = report_output.parent()
      && !parent.as_os_str().is_empty()
    {
      fs::create_dir_all(parent)
        .map_err(Error::from_std)
        .wrap_context("create report output directory failed")
        .wrap_context_with(|| format!("dir={}", parent.display()))?;
    }

    fs::write(report_output, report)
      .map_err(Error::from_std)
      .wrap_context("write rule report failed")
      .wrap_context_with(|| format!("path={}", report_output.display()))?;
  } else {
    print!("{report}");
    if !report.ends_with('\n') {
      println!();
    }
  }

  Ok(())
}

fn append_matched_rules(output: &mut String, service: &str, matched_rules: &[TypeOverrideHit]) {
  writeln!(output, "matched rules for {service}:").expect("write string");

  if matched_rules.is_empty() {
    writeln!(output, "  (no matched rules)").expect("write string");
    return;
  }

  for hit in matched_rules {
    writeln!(
      output,
      "  {} -> {} [{}:{}]",
      hit.field_path,
      hit.rust_type,
      hit.rule_source.as_str(),
      hit.rule_key
    )
    .expect("write string");
  }
}

fn append_unused_rules(output: &mut String, unused_rules: &[TypeOverrideRule]) {
  writeln!(output, "unused rules:").expect("write string");

  if unused_rules.is_empty() {
    writeln!(output, "  (no unused rules)").expect("write string");
    return;
  }

  for rule in unused_rules {
    writeln!(
      output,
      "  {}:{} -> {}",
      rule.rule_source.as_str(),
      rule.rule_key,
      rule.rust_type
    )
    .expect("write string");
  }
}
