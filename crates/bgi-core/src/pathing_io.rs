use super::*;

pub fn read_pathing_task(path: impl AsRef<Path>) -> Result<PathingTask> {
    let path = path.as_ref();
    let text = read_pathing_json_with_control(path)?;
    let mut task: PathingTask =
        json5::from_str(&text).map_err(|err| BgiError::json(Some(path), err.to_string()))?;
    task.file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned);
    task.full_path = Some(path.to_path_buf());
    apply_file_task_defaults(&mut task);
    Ok(task)
}

fn apply_file_task_defaults(task: &mut PathingTask) {
    for position in &mut task.positions {
        position.point_ext_params.enable_monster_loot_split = task.info.enable_monster_loot_split;
    }
}

pub fn read_pathing_json_with_control(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let original_text = fs::read_to_string(path).map_err(|source| BgiError::io(path, source))?;
    let Some(parent) = path.parent() else {
        return Ok(original_text);
    };

    let control_path = parent.join(CONTROL_FILE_NAME);
    if !control_path.exists() {
        return Ok(original_text);
    }

    let control = read_control_json(&control_path)?;
    let mut original: Value = json5::from_str(&original_text)
        .map_err(|err| BgiError::json(Some(path), err.to_string()))?;
    let name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    merge_pathing_json(&control, &mut original, name);
    serde_json::to_string_pretty(&original)
        .map_err(|err| BgiError::json(Some(path), err.to_string()))
}

pub(super) fn read_control_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).map_err(|source| BgiError::io(path, source))?;
    let value: Value =
        json5::from_str(&text).map_err(|err| BgiError::json(Some(path), err.to_string()))?;

    if let Some(reference) = value.get("ref").and_then(Value::as_str) {
        let referenced = path
            .parent()
            .map(|parent| parent.join(reference))
            .unwrap_or_else(|| PathBuf::from(reference));
        return read_control_json(&referenced);
    }

    Ok(value)
}

pub(super) fn merge_pathing_json(control: &Value, original: &mut Value, name: &str) {
    if let Some(global_cover) = control.get("global_cover") {
        merge_value(global_cover, original);
    }

    let Some(json_list) = control.get("json_list").and_then(Value::as_array) else {
        return;
    };

    for item in json_list {
        let item_name = item.get("name").and_then(Value::as_str);
        if item_name != Some(name) {
            continue;
        }

        if let Some(cover) = item.get("cover") {
            merge_value(cover, original);
        }
        break;
    }
}

pub(super) fn merge_value(control: &Value, target: &mut Value) {
    let Value::Object(control) = control else {
        *target = control.clone();
        return;
    };

    if !target.is_object() {
        *target = Value::Object(control.clone());
    }

    let Some(target) = target.as_object_mut() else {
        return;
    };

    let mut skip_keys = HashSet::new();
    process_special_instructions(control, target, &mut skip_keys);

    for (key, control_value) in control {
        if skip_keys.contains(key) {
            continue;
        }

        if control_value.is_object() {
            if let Some(Value::Object(_)) = target.get(key) {
                if let Some(target_value) = target.get_mut(key) {
                    merge_value(control_value, target_value);
                    continue;
                }
            }
        }

        target.insert(key.clone(), control_value.clone());
    }
}

pub(super) fn process_special_instructions(
    control: &Map<String, Value>,
    target: &mut Map<String, Value>,
    skip_keys: &mut HashSet<String>,
) {
    if let Some(Value::Array(keys)) = control.get("_obj_cover") {
        for item in keys {
            let Some(prop_name) = item.as_str() else {
                continue;
            };
            if let Some(value) = control.get(prop_name) {
                target.insert(prop_name.to_string(), value.clone());
                skip_keys.insert(prop_name.to_string());
            }
        }
        skip_keys.insert("_obj_cover".to_string());
    }

    if let Some(Value::Array(keys)) = control.get("_arr_add") {
        for item in keys {
            let Some(prop_name) = item.as_str() else {
                continue;
            };
            let Some(Value::Array(source_array)) = control.get(prop_name) else {
                continue;
            };

            let merged = match target.get(prop_name).and_then(Value::as_array) {
                Some(target_array) => merge_arrays(source_array, target_array),
                None => Value::Array(source_array.clone()),
            };
            target.insert(prop_name.to_string(), merged);
            skip_keys.insert(prop_name.to_string());
        }
        skip_keys.insert("_arr_add".to_string());
    }
}

pub(super) fn merge_arrays(source: &[Value], target: &[Value]) -> Value {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for item in target.iter().chain(source.iter()) {
        let key = serde_json::to_string(item).unwrap_or_else(|_| format!("{item:?}"));
        if seen.insert(key) {
            result.push(item.clone());
        }
    }

    Value::Array(result)
}
