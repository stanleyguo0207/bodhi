use bodhi_config::merge::deep_merge;
use toml::Value;

#[test]
fn deep_merge_should_merge_tables_recursively() {
  let mut base: Value = toml::from_str(
    r#"
    [log]
    level = "INFO"
    [net]
    timeout = 10
    "#,
  )
  .expect("parse base toml");

  let overlay: Value = toml::from_str(
    r#"
    [log]
    level = "DEBUG"
    format = "json"
    "#,
  )
  .expect("parse overlay toml");

  deep_merge(&mut base, &overlay);

  assert_eq!(base["log"]["level"].as_str(), Some("DEBUG"));
  assert_eq!(base["log"]["format"].as_str(), Some("json"));
  assert_eq!(base["net"]["timeout"].as_integer(), Some(10));
}

#[test]
fn deep_merge_should_replace_arrays() {
  let mut base: Value = toml::from_str("items = [1, 2, 3]").expect("parse base toml");
  let overlay: Value = toml::from_str("items = [4, 5]").expect("parse overlay toml");

  deep_merge(&mut base, &overlay);

  let items = base["items"].as_array().expect("items should be array");
  assert_eq!(items.len(), 2);
  assert_eq!(items[0].as_integer(), Some(4));
  assert_eq!(items[1].as_integer(), Some(5));
}
