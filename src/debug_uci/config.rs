//! One flattened description of the actual serialized EngineConfig. New config
//! fields automatically appear in UCI and in clients that consume the schema.
use crate::engine::{MAX_PLY, configs::EngineConfig};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize)]
pub struct ConfigField {
    pub path: String,
    pub kind: &'static str,
    pub default: Value,
    pub min: Option<i64>,
    pub max: Option<i64>,
}

pub fn fields() -> Vec<ConfigField> {
    fn visit(path: &str, value: &Value, output: &mut Vec<ConfigField>) {
        if let Value::Object(object) = value {
            for (key, value) in object {
                let path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                visit(&path, value, output);
            }
            return;
        }
        let kind = if matches!(
            path,
            "limits.max_nodes" | "limits.soft_time_limit_ms" | "limits.hard_time_limit_ms"
        ) {
            "optional_integer"
        } else {
            match value {
                Value::Bool(_) => "bool",
                Value::Number(n) if n.is_f64() => "float",
                Value::Number(_) => "integer",
                Value::Null => "optional_integer",
                _ => "string",
            }
        };
        let (min, max) = match path {
            "tt_size" => (Some(1), Some(4096)),
            "limits.max_depth" => (Some(1), Some((MAX_PLY - 1) as i64)),
            "limits.max_q_depth" => (Some(0), Some((MAX_PLY - 1) as i64)),
            "search.fut.margin" | "search.fut.history_margin" => (Some(0), Some(u16::MAX as i64)),
            p if p.ends_with(".min_depth")
                || p.ends_with(".max_depth")
                || p.ends_with(".minimum_depth") =>
            {
                (Some(0), Some((MAX_PLY - 1) as i64))
            }
            "search.lmr.history_scale"
            | "search.aspiration.initial_window"
            | "search.aspiration.max_window" => (Some(1), Some(1_000_000)),
            "search.aspiration.growth_factor" => (Some(2), Some(100)),
            _ if kind == "integer" => (Some(i32::MIN as i64), Some(i32::MAX as i64)),
            _ => (None, None),
        };
        output.push(ConfigField {
            path: path.into(),
            kind,
            default: value.clone(),
            min,
            max,
        });
    }
    let mut result = Vec::new();
    visit(
        "",
        &serde_json::to_value(EngineConfig::default()).expect("serializable defaults"),
        &mut result,
    );
    result
}

pub fn option_line(field: &ConfigField) -> String {
    let default = value_text(&field.default);
    match field.kind {
        "bool" => format!("option name {} type check default {default}", field.path),
        "integer" => format!(
            "option name {} type spin default {default} min {} max {}",
            field.path,
            field.min.unwrap(),
            field.max.unwrap()
        ),
        _ => format!("option name {} type string default {default}", field.path),
    }
}

pub fn value_text(value: &Value) -> String {
    match value {
        Value::Null => "<none>".into(),
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

pub fn set_option(config: &mut EngineConfig, name: &str, input: &str) -> Result<(), String> {
    let name = if name.eq_ignore_ascii_case("Hash") {
        "tt_size"
    } else {
        name
    };
    let field = fields()
        .into_iter()
        .find(|f| f.path.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("unknown configuration option `{name}`"))?;
    let value = match field.kind {
        "bool" => Value::Bool(match input.to_ascii_lowercase().as_str() {
            "true" => true,
            "false" => false,
            _ => return Err(format!("{} expects true or false", field.path)),
        }),
        "integer" => {
            let number = input
                .parse::<i64>()
                .map_err(|_| format!("{} expects an integer", field.path))?;
            if field.min.is_some_and(|n| number < n) || field.max.is_some_and(|n| number > n) {
                return Err(format!(
                    "{} must be between {} and {}",
                    field.path,
                    field.min.unwrap(),
                    field.max.unwrap()
                ));
            }
            Value::from(number)
        }
        "float" => {
            let number = input
                .parse::<f64>()
                .map_err(|_| format!("{} expects a decimal number", field.path))?;
            if !number.is_finite() {
                return Err("configuration numbers must be finite".into());
            }
            if field.path == "search.lmr.divisor" && number <= 0.0 {
                return Err("LMR divisor must be positive".into());
            }
            Value::from(number)
        }
        "optional_integer" => {
            if matches!(
                input.to_ascii_lowercase().as_str(),
                "<none>" | "none" | "null" | "<empty>" | ""
            ) {
                Value::Null
            } else {
                Value::from(input.parse::<u64>().map_err(|_| {
                    format!("{} expects a nonnegative integer or <none>", field.path)
                })?)
            }
        }
        _ => Value::String(input.into()),
    };
    let mut document = serde_json::to_value(*config).map_err(|e| e.to_string())?;
    let mut target = &mut document;
    for part in field.path.split('.') {
        target = target.get_mut(part).ok_or("missing configuration field")?;
    }
    *target = value;
    *config = serde_json::from_value(document).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn validate(config: &EngineConfig) -> Result<(), String> {
    if config.search.aspiration.initial_window <= 0
        || config.search.aspiration.growth_factor < 2
        || config.search.aspiration.max_window < config.search.aspiration.initial_window
    {
        return Err("aspiration requires initial_window > 0, growth_factor >= 2, max_window >= initial_window".into());
    }
    if !(0..=10_000).contains(&config.search.aspiration.mate_margin) {
        return Err("aspiration mate_margin must be between 0 and 10000".into());
    }
    if config.search.lmr.history_scale <= 0
        || !config.search.lmr.divisor.is_finite()
        || config.search.lmr.divisor <= 0.0
        || !config.search.lmr.base.is_finite()
    {
        return Err(
            "LMR requires a positive divisor and history_scale, and finite parameters".into(),
        );
    }
    if !(1..=4096).contains(&config.tt_size)
        || config.limits.max_depth == 0
        || config.limits.max_depth as usize >= MAX_PLY
        || config.limits.max_q_depth as usize >= MAX_PLY
    {
        return Err("invalid hash size or search depth".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fields_follow_defaults_and_changes_are_typed() {
        let mut c = EngineConfig::default();
        let original = fields();
        assert!(
            original
                .iter()
                .any(|f| f.path == "search.lmr.divisor" && f.kind == "float")
        );
        set_option(&mut c, "SEARCH.LMR.DIVISOR", "3.25").unwrap();
        set_option(&mut c, "search.null_move.enabled", "false").unwrap();
        set_option(&mut c, "limits.max_nodes", "10000").unwrap();
        assert_eq!(c.search.lmr.divisor, 3.25);
        assert!(!c.search.null_move.enabled);
        assert_eq!(c.limits.max_nodes, Some(10000));
        assert!(set_option(&mut c, "search.lmr.divisor", "0").is_err());
        assert!(set_option(&mut c, "search.lmr.history_scale", "0").is_err());
        assert!(set_option(&mut c, "search.lmr.base", "NaN").is_err());
        assert_eq!(c.search.lmr.divisor, 3.25);
    }
}
