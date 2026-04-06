//! Rust 配置结构代码生成模块

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use bodhi_error::prelude::*;
use serde::{Deserialize, Serialize};
use syn::Type;
use toml::Value;

use crate::errcode::configerr::*;

const MERGED_MODULE_NAME: &str = "merged";
const INFRA_MODULE_NAME: &str = "infra";
const SERVICE_MODULE_NAME: &str = "service";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RustCodegenResult {
  pub content: String,
  pub matched_rules: Vec<TypeOverrideHit>,
  pub unused_rules: Vec<TypeOverrideRule>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TypeOverrideHit {
  pub field_path: String,
  pub rust_type: String,
  pub rule_key: String,
  pub rule_source: TypeOverrideSource,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TypeOverrideRule {
  pub rust_type: String,
  pub rule_key: String,
  pub rule_source: TypeOverrideSource,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeOverrideSource {
  ExactPath,
  GlobPath,
  Field,
  Suffix,
}

impl TypeOverrideSource {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::ExactPath => "exact-path",
      Self::GlobPath => "glob-path",
      Self::Field => "field",
      Self::Suffix => "suffix",
    }
  }
}

#[derive(Clone, Debug)]
struct ResolvedTypeOverride {
  rust_type: String,
  rule_key: String,
  rule_source: TypeOverrideSource,
}

#[derive(Clone, Debug)]
pub struct RustCodegenOptions {
  pub root_struct_name: String,
  pub type_overrides: TypeOverrideRules,
}

impl Default for RustCodegenOptions {
  fn default() -> Self {
    Self {
      root_struct_name: String::from("Config"),
      type_overrides: TypeOverrideRules::default(),
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TypeOverrideRules {
  #[serde(default)]
  pub field_types: BTreeMap<String, String>,
  #[serde(default)]
  pub path_types: BTreeMap<String, String>,
  #[serde(default)]
  pub suffix_types: BTreeMap<String, String>,
}

impl TypeOverrideRules {
  pub fn from_file(path: &Path) -> Result<Self> {
    let content = fs::read_to_string(path)
      .map_err(Error::from_std)
      .wrap_context("read type override rules file failed")
      .wrap_context_with(|| format!("path={}", path.display()))?;

    let rules: Self = toml::from_str(&content)
      .map_err(Error::from_std)
      .wrap_context("parse type override rules file failed")
      .wrap_context_with(|| format!("path={}", path.display()))?;
    rules.validate()?;
    Ok(rules)
  }

  pub fn defined_rules(&self) -> Vec<TypeOverrideRule> {
    let mut rules = Vec::new();

    for (key, ty) in &self.path_types {
      rules.push(TypeOverrideRule {
        rust_type: ty.clone(),
        rule_key: key.clone(),
        rule_source: path_rule_source(key),
      });
    }

    for (key, ty) in &self.field_types {
      rules.push(TypeOverrideRule {
        rust_type: ty.clone(),
        rule_key: key.clone(),
        rule_source: TypeOverrideSource::Field,
      });
    }

    for (key, ty) in &self.suffix_types {
      rules.push(TypeOverrideRule {
        rust_type: ty.clone(),
        rule_key: key.clone(),
        rule_source: TypeOverrideSource::Suffix,
      });
    }

    rules
  }

  pub fn find_unused_rules(&self, matched_rules: &[TypeOverrideHit]) -> Vec<TypeOverrideRule> {
    let matched_rule_keys: BTreeSet<_> = matched_rules
      .iter()
      .map(|hit| (hit.rule_source, hit.rule_key.clone()))
      .collect();

    self
      .defined_rules()
      .into_iter()
      .filter(|rule| !matched_rule_keys.contains(&(rule.rule_source, rule.rule_key.clone())))
      .collect()
  }

  fn resolve(&self, path: &[String], field_name: &str) -> Option<ResolvedTypeOverride> {
    let full_path = join_segments(path, field_name);

    if let Some(ty) = self.path_types.get(&full_path) {
      return Some(ResolvedTypeOverride {
        rust_type: ty.clone(),
        rule_key: full_path,
        rule_source: TypeOverrideSource::ExactPath,
      });
    }

    if let Some(ty) = self.resolve_glob_path_type(&full_path) {
      return Some(ty);
    }

    if let Some(ty) = self.field_types.get(field_name) {
      return Some(ResolvedTypeOverride {
        rust_type: ty.clone(),
        rule_key: field_name.to_string(),
        rule_source: TypeOverrideSource::Field,
      });
    }

    self
      .suffix_types
      .iter()
      .filter(|(suffix, _)| field_name.ends_with(suffix.as_str()))
      .max_by_key(|(suffix, _)| suffix.len())
      .map(|(suffix, ty)| ResolvedTypeOverride {
        rust_type: ty.clone(),
        rule_key: suffix.clone(),
        rule_source: TypeOverrideSource::Suffix,
      })
  }

  fn validate(&self) -> Result<()> {
    validate_rule_map(&self.field_types, "field_types")?;
    validate_rule_map(&self.path_types, "path_types")?;
    validate_rule_map(&self.suffix_types, "suffix_types")?;
    Ok(())
  }

  fn resolve_glob_path_type(&self, full_path: &str) -> Option<ResolvedTypeOverride> {
    self
      .path_types
      .iter()
      .filter(|(pattern, _)| pattern.contains('*') && glob_path_matches(pattern, full_path))
      .max_by_key(|(pattern, _)| glob_specificity(pattern))
      .map(|(pattern, ty)| ResolvedTypeOverride {
        rust_type: ty.clone(),
        rule_key: pattern.clone(),
        rule_source: TypeOverrideSource::GlobPath,
      })
  }
}

pub fn render_rust_types(value: &Value, options: &RustCodegenOptions) -> Result<String> {
  Ok(render_rust_types_report(value, options)?.content)
}

pub fn render_layered_rust_types(
  infra: &Value,
  service: &Value,
  merged: &Value,
  options: &RustCodegenOptions,
) -> Result<String> {
  Ok(render_layered_rust_types_report(infra, service, merged, options)?.content)
}

pub fn render_rust_types_report(
  value: &Value,
  options: &RustCodegenOptions,
) -> Result<RustCodegenResult> {
  let root_table = value
    .as_table()
    .ok_or_else(|| Error::new(CONFIGERR_INVALIDSTRUCTURE))
    .wrap_context("resolved config root must be a table")?;

  let root_struct_name = sanitize_type_name(&options.root_struct_name);
  let mut generator = Generator {
    type_overrides: options.type_overrides.clone(),
    ..Default::default()
  };
  generator.used_struct_names.insert(root_struct_name.clone());
  generator.visit_table(root_struct_name.clone(), &[], root_table)?;
  let content = generator.render(&root_struct_name);
  let matched_rules = generator.matched_rules;
  let unused_rules = options.type_overrides.find_unused_rules(&matched_rules);

  Ok(RustCodegenResult {
    content,
    matched_rules,
    unused_rules,
  })
}

pub fn render_layered_rust_types_report(
  infra: &Value,
  service: &Value,
  merged: &Value,
  options: &RustCodegenOptions,
) -> Result<RustCodegenResult> {
  let merged_module = generate_module(merged, options, &options.root_struct_name)?;
  let infra_module = generate_module(infra, options, "Config")?;
  let service_module = generate_module(service, options, "Config")?;

  let content = render_layered_modules(&merged_module, &infra_module, &service_module);
  let matched_rules = unique_hits(
    merged_module
      .matched_rules
      .into_iter()
      .chain(infra_module.matched_rules)
      .chain(service_module.matched_rules),
  );
  let unused_rules = options.type_overrides.find_unused_rules(&matched_rules);

  Ok(RustCodegenResult {
    content,
    matched_rules,
    unused_rules,
  })
}

pub fn write_rust_types(output_path: &Path, content: &str) -> Result<()> {
  if let Some(parent) = output_path.parent() {
    fs::create_dir_all(parent)
      .map_err(Error::from_std)
      .wrap_context("create rust output directory failed")
      .wrap_context_with(|| format!("dir={}", parent.display()))?;
  }

  fs::write(output_path, content)
    .map_err(Error::from_std)
    .wrap_context("write rust output file failed")
    .wrap_context_with(|| format!("path={}", output_path.display()))
}

#[derive(Debug, Default)]
struct Generator {
  definitions: Vec<StructDefinition>,
  matched_rules: Vec<TypeOverrideHit>,
  type_overrides: TypeOverrideRules,
  used_struct_names: BTreeSet<String>,
}

impl Generator {
  fn visit_table(
    &mut self,
    struct_name: String,
    path: &[String],
    table: &toml::map::Map<String, Value>,
  ) -> Result<()> {
    let mut fields = Vec::new();
    let mut keys: Vec<_> = table.keys().cloned().collect();
    keys.sort();

    for key in keys {
      let value = table.get(&key).expect("table key should exist");
      let (field_name, rename) = sanitize_field_name(&key);
      let overridden_type = self.type_overrides.resolve(path, &key);
      let field_path = join_segments(path, &key);

      let field_type = match overridden_type {
        Some(override_hit) => {
          self.matched_rules.push(TypeOverrideHit {
            field_path,
            rust_type: override_hit.rust_type.clone(),
            rule_key: override_hit.rule_key.clone(),
            rule_source: override_hit.rule_source,
          });
          override_hit.rust_type
        }
        None => match value {
          Value::Table(child_table) => {
            let child_struct_name = self.allocate_struct_name(path, &key, "Config");
            let mut child_path = path.to_vec();
            child_path.push(key.clone());
            self.visit_table(child_struct_name.clone(), &child_path, child_table)?;
            child_struct_name
          }
          Value::Array(items) => self.array_type(path, &key, items)?,
          _ => scalar_type(&key, value),
        },
      };

      fields.push(FieldDefinition {
        name: field_name,
        rename,
        ty: field_type,
      });
    }

    self.definitions.push(StructDefinition {
      name: struct_name,
      fields,
    });

    Ok(())
  }

  fn array_type(&mut self, path: &[String], key: &str, items: &[Value]) -> Result<String> {
    if items.is_empty() {
      return Ok(String::from("Vec<bodhi_config::toml::Value>"));
    }

    let first = &items[0];
    let item_type = match first {
      Value::Table(child_table) => {
        for item in items.iter().skip(1) {
          if !matches!(item, Value::Table(_)) {
            return Err(
              Error::new(CONFIGERR_CODEGENFAILED)
                .wrap_context("array contains mixed value kinds")
                .wrap_context_with(|| format!("path={}", join_path(path, key))),
            );
          }
        }

        let child_struct_name = self.allocate_struct_name(path, key, "Item");
        let mut child_path = path.to_vec();
        child_path.push(key.to_string());
        self.visit_table(child_struct_name.clone(), &child_path, child_table)?;
        child_struct_name
      }
      _ => {
        for item in items.iter().skip(1) {
          if !same_scalar_kind(first, item) {
            return Err(
              Error::new(CONFIGERR_CODEGENFAILED)
                .wrap_context("array contains mixed scalar types")
                .wrap_context_with(|| format!("path={}", join_path(path, key))),
            );
          }
        }

        scalar_type(key, first)
      }
    };

    Ok(format!("Vec<{item_type}>"))
  }

  fn allocate_struct_name(&mut self, path: &[String], key: &str, suffix: &str) -> String {
    let mut segments = path.to_vec();
    segments.push(key.to_string());
    let base_name = format!("{}{}", to_pascal_case(&segments.join("_")), suffix);
    allocate_unique_name(&mut self.used_struct_names, &sanitize_type_name(&base_name))
  }

  fn render(&self, root_struct_name: &str) -> String {
    let mut definitions = self.definitions.clone();
    definitions.sort_by(|left, right| {
      if left.name == root_struct_name {
        std::cmp::Ordering::Less
      } else if right.name == root_struct_name {
        std::cmp::Ordering::Greater
      } else {
        left.name.cmp(&right.name)
      }
    });

    let mut output = String::from("use serde::Deserialize;\n\n");
    for (index, definition) in definitions.iter().enumerate() {
      if index > 0 {
        output.push('\n');
      }

      output.push_str("#[derive(Debug, Deserialize)]\n");
      output.push_str(&format!("pub struct {} {{\n", definition.name));
      for field in &definition.fields {
        if let Some(rename) = &field.rename {
          output.push_str(&format!("  #[serde(rename = \"{}\")]\n", rename));
        }
        output.push_str(&format!("  pub {}: {},\n", field.name, field.ty));
      }
      output.push_str("}\n");
    }

    output
  }
}

#[derive(Debug)]
struct GeneratedModule {
  root_struct_name: String,
  definitions: Vec<StructDefinition>,
  matched_rules: Vec<TypeOverrideHit>,
}

#[derive(Clone, Debug)]
struct StructDefinition {
  name: String,
  fields: Vec<FieldDefinition>,
}

#[derive(Clone, Debug)]
struct FieldDefinition {
  name: String,
  rename: Option<String>,
  ty: String,
}

fn generate_module(
  value: &Value,
  options: &RustCodegenOptions,
  root_struct_name: &str,
) -> Result<GeneratedModule> {
  let root_table = value
    .as_table()
    .ok_or_else(|| Error::new(CONFIGERR_INVALIDSTRUCTURE))
    .wrap_context("resolved config root must be a table")?;

  let root_struct_name = sanitize_type_name(root_struct_name);
  let mut generator = Generator {
    type_overrides: options.type_overrides.clone(),
    ..Default::default()
  };
  generator.used_struct_names.insert(root_struct_name.clone());
  generator.visit_table(root_struct_name.clone(), &[], root_table)?;

  Ok(GeneratedModule {
    root_struct_name,
    definitions: generator.definitions,
    matched_rules: generator.matched_rules,
  })
}

fn render_layered_modules(
  merged: &GeneratedModule,
  infra: &GeneratedModule,
  service: &GeneratedModule,
) -> String {
  let mut output = String::from("use serde::Deserialize;\n\n");
  output.push_str(&render_module(MERGED_MODULE_NAME, merged));
  output.push('\n');
  output.push('\n');
  output.push_str(&render_module(INFRA_MODULE_NAME, infra));
  output.push('\n');
  output.push('\n');
  output.push_str(&render_module(SERVICE_MODULE_NAME, service));
  output.push('\n');
  output.push('\n');
  output.push_str("pub use merged::Config;\n");

  output
}

fn render_module(module_name: &str, module: &GeneratedModule) -> String {
  let mut output = format!("pub mod {module_name} {{\n  use super::*;\n\n");
  let mut definitions = module.definitions.clone();
  definitions.sort_by(|left, right| {
    if left.name == module.root_struct_name {
      std::cmp::Ordering::Less
    } else if right.name == module.root_struct_name {
      std::cmp::Ordering::Greater
    } else {
      left.name.cmp(&right.name)
    }
  });

  for (index, definition) in definitions.iter().enumerate() {
    if index > 0 {
      output.push('\n');
    }

    output.push_str("  #[derive(Debug, Deserialize)]\n");
    output.push_str(&format!("  pub struct {} {{\n", definition.name));
    for field in &definition.fields {
      if let Some(rename) = &field.rename {
        output.push_str(&format!("    #[serde(rename = \"{}\")]\n", rename));
      }
      output.push_str(&format!("    pub {}: {},\n", field.name, field.ty));
    }
    output.push_str("  }\n");
  }

  output.push('}');
  output
}

fn unique_hits(hits: impl IntoIterator<Item = TypeOverrideHit>) -> Vec<TypeOverrideHit> {
  let mut seen = BTreeSet::new();
  let mut unique = Vec::new();

  for hit in hits {
    let key = (
      hit.field_path.clone(),
      hit.rust_type.clone(),
      hit.rule_key.clone(),
      hit.rule_source,
    );
    if seen.insert(key) {
      unique.push(hit);
    }
  }

  unique
}

fn scalar_type(field_name: &str, value: &Value) -> String {
  match value {
    Value::String(_) => String::from("String"),
    Value::Integer(number) => infer_integer_type(field_name, *number),
    Value::Float(_) => String::from("f64"),
    Value::Boolean(_) => String::from("bool"),
    Value::Datetime(_) => String::from("bodhi_config::toml::value::Datetime"),
    Value::Array(_) => String::from("Vec<bodhi_config::toml::Value>"),
    Value::Table(_) => String::from("bodhi_config::toml::Value"),
  }
}

fn infer_integer_type(field_name: &str, number: i64) -> String {
  if number >= 0 {
    if field_name.ends_with("_port") && number <= i64::from(u16::MAX) {
      return String::from("u16");
    }

    if field_name.ends_with("_ms") {
      return String::from("u64");
    }

    return String::from("u64");
  }

  String::from("i64")
}

fn sanitize_field_name(name: &str) -> (String, Option<String>) {
  let sanitized = sanitize_identifier(name);
  let rename = if sanitized == name || raw_keyword_name(name) == sanitized {
    None
  } else {
    Some(name.to_string())
  };
  (sanitized, rename)
}

fn sanitize_type_name(name: &str) -> String {
  let pascal = to_pascal_case(name);
  if pascal.is_empty() {
    String::from("Config")
  } else if pascal.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
    format!("X{pascal}")
  } else {
    pascal
  }
}

fn sanitize_identifier(name: &str) -> String {
  let mut output = String::new();
  for ch in name.chars() {
    if ch.is_ascii_alphanumeric() || ch == '_' {
      output.push(ch);
    } else {
      output.push('_');
    }
  }

  if output.is_empty() {
    output.push('_');
  }

  if output.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
    output.insert(0, '_');
  }

  if is_keyword(&output) {
    return raw_keyword_name(&output);
  }

  output
}

fn raw_keyword_name(name: &str) -> String {
  format!("r#{name}")
}

fn is_keyword(name: &str) -> bool {
  matches!(
    name,
    "as"
      | "break"
      | "const"
      | "continue"
      | "crate"
      | "else"
      | "enum"
      | "extern"
      | "false"
      | "fn"
      | "for"
      | "if"
      | "impl"
      | "in"
      | "let"
      | "loop"
      | "match"
      | "mod"
      | "move"
      | "mut"
      | "pub"
      | "ref"
      | "return"
      | "self"
      | "Self"
      | "static"
      | "struct"
      | "super"
      | "trait"
      | "true"
      | "type"
      | "unsafe"
      | "use"
      | "where"
      | "while"
      | "async"
      | "await"
      | "dyn"
  )
}

fn to_pascal_case(value: &str) -> String {
  value
    .split(|ch: char| !ch.is_ascii_alphanumeric())
    .filter(|segment| !segment.is_empty())
    .map(|segment| {
      let mut chars = segment.chars();
      let Some(first) = chars.next() else {
        return String::new();
      };

      let mut output = String::new();
      output.push(first.to_ascii_uppercase());
      output.extend(chars);
      output
    })
    .collect()
}

fn allocate_unique_name(used_names: &mut BTreeSet<String>, base_name: &str) -> String {
  if used_names.insert(base_name.to_string()) {
    return base_name.to_string();
  }

  let mut index = 2usize;
  loop {
    let candidate = format!("{base_name}{index}");
    if used_names.insert(candidate.clone()) {
      return candidate;
    }
    index += 1;
  }
}

fn join_path(path: &[String], key: &str) -> String {
  if path.is_empty() {
    key.to_string()
  } else {
    format!("{}.{}", path.join("."), key)
  }
}

fn same_scalar_kind(left: &Value, right: &Value) -> bool {
  matches!(
    (left, right),
    (Value::String(_), Value::String(_))
      | (Value::Integer(_), Value::Integer(_))
      | (Value::Float(_), Value::Float(_))
      | (Value::Boolean(_), Value::Boolean(_))
      | (Value::Datetime(_), Value::Datetime(_))
  )
}

fn validate_rule_map(rules: &BTreeMap<String, String>, section: &str) -> Result<()> {
  for (key, ty) in rules {
    if key.trim().is_empty() {
      return Err(
        Error::new(CONFIGERR_CODEGENFAILED)
          .wrap_context("type override rule key must not be empty")
          .wrap_context_with(|| format!("section={section}")),
      );
    }

    if ty.trim().is_empty() {
      return Err(
        Error::new(CONFIGERR_CODEGENFAILED)
          .wrap_context("type override rule target type must not be empty")
          .wrap_context_with(|| format!("section={section} key={key}")),
      );
    }

    validate_type_expr(ty, section, key)?;

    if section == "path_types" && key.contains(' ') {
      return Err(
        Error::new(CONFIGERR_CODEGENFAILED)
          .wrap_context("path_types rule key must not contain spaces")
          .wrap_context_with(|| format!("section={section} key={key}")),
      );
    }
  }

  Ok(())
}

fn validate_type_expr(ty: &str, section: &str, key: &str) -> Result<()> {
  syn::parse_str::<Type>(ty)
    .map_err(Error::from_std)
    .wrap_context("invalid Rust type expression in type override rule")
    .wrap_context_with(|| format!("section={section} key={key} ty={ty}"))?;

  Ok(())
}

fn join_segments(path: &[String], field_name: &str) -> String {
  if path.is_empty() {
    field_name.to_string()
  } else {
    format!("{}.{}", path.join("."), field_name)
  }
}

fn path_rule_source(key: &str) -> TypeOverrideSource {
  if key.contains('*') {
    TypeOverrideSource::GlobPath
  } else {
    TypeOverrideSource::ExactPath
  }
}

fn glob_path_matches(pattern: &str, full_path: &str) -> bool {
  let pattern_segments: Vec<_> = pattern.split('.').collect();
  let path_segments: Vec<_> = full_path.split('.').collect();
  glob_segments_match(&pattern_segments, &path_segments)
}

fn glob_segments_match(pattern: &[&str], path: &[&str]) -> bool {
  match (pattern.split_first(), path.split_first()) {
    (None, None) => true,
    (None, Some(_)) => false,
    (Some((&"**", pattern_tail)), _) => {
      glob_segments_match(pattern_tail, path)
        || (!path.is_empty() && glob_segments_match(pattern, &path[1..]))
    }
    (Some((pattern_head, pattern_tail)), Some((path_head, path_tail))) => {
      segment_matches(pattern_head, path_head) && glob_segments_match(pattern_tail, path_tail)
    }
    (Some(_), None) => false,
  }
}

fn segment_matches(pattern: &str, value: &str) -> bool {
  if pattern == "*" {
    return true;
  }

  if !pattern.contains('*') {
    return pattern == value;
  }

  wildcard_match(pattern, value)
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
  let parts: Vec<_> = pattern.split('*').collect();
  if parts.len() == 1 {
    return pattern == value;
  }

  let starts_with_wildcard = pattern.starts_with('*');
  let ends_with_wildcard = pattern.ends_with('*');
  let mut cursor = 0usize;

  for (index, part) in parts.iter().enumerate() {
    if part.is_empty() {
      continue;
    }

    if index == 0 && !starts_with_wildcard {
      if !value[cursor..].starts_with(part) {
        return false;
      }
      cursor += part.len();
      continue;
    }

    if index == parts.len() - 1 && !ends_with_wildcard {
      if let Some(found_at) = value[cursor..].rfind(part) {
        let absolute = cursor + found_at;
        return absolute + part.len() == value.len();
      }
      return false;
    }

    if let Some(found_at) = value[cursor..].find(part) {
      cursor += found_at + part.len();
    } else {
      return false;
    }
  }

  true
}

fn glob_specificity(pattern: &str) -> (usize, usize) {
  let literal_chars = pattern.chars().filter(|ch| *ch != '*').count();
  let segments = pattern.split('.').count();
  (literal_chars, segments)
}
