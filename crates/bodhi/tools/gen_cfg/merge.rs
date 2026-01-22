use bodhi::{BODHIERR_CONFIGKEYNOTFOUND, BODHIERR_INVALIDCONFIGTYPE, Error, Result};
use toml::Value;

fn merge_value(base: &mut Value, overlay: Value) -> Result<()> {
  match (&mut *base, overlay) {
    (Value::Table(base_tbl), Value::Table(overlay_tbl)) => {
      for (k, v) in overlay_tbl {
        match base_tbl.get_mut(&k) {
          Some(b) => merge_value(b, v)?,
          None => {
            return Err(
              Error::new(&BODHIERR_CONFIGKEYNOTFOUND)
                .wrap_context_with(|| format!("k:{:?} v:{:?}", k, v)),
            );
          }
        }
      }

      Ok(())
    }

    (Value::Array(_), Value::Array(arr)) => {
      *base = Value::Array(arr);
      Ok(())
    }

    (b, o) => {
      if std::mem::discriminant(b) != std::mem::discriminant(&o) {
        return Err(
          Error::new(&BODHIERR_INVALIDCONFIGTYPE)
            .wrap_context_with(|| format!("b:{:?} o:{:?}", b, o)),
        );
      }

      *b = o;
      Ok(())
    }
  }
}

pub fn merge_configs(
  base: impl IntoIterator<Item = (String, Value)>,
  overlay: impl IntoIterator<Item = (String, Value)>,
) -> Result<Vec<(String, Value)>> {
  let mut result = base
    .into_iter()
    .collect::<std::collections::HashMap<_, _>>();

  for (k, v) in overlay {
    match result.get_mut(&k) {
      Some(base_val) => {
        merge_value(base_val, v)?;
      }
      None => {
        // return Err(
        //   Error::new(&BODHIERR_CONFIGKEYNOTFOUND)
        //     .wrap_context_with(|| format!("k:{:?} v:{:?}", k, v)),
        // );
        result.insert(k, v);
      }
    }
  }

  Ok(result.into_iter().collect())
}
