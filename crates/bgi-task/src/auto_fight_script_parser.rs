use super::*;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn parse_combat_script_file(
    path: &Path,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<CombatScriptPlan> {
    let context =
        fs::read_to_string(path).map_err(|error| TaskError::CombatStrategy(error.to_string()))?;
    let mut script = parse_combat_script_context_with_catalog(&context, true, catalog)?;
    script.path = Some(path.to_path_buf());
    script.name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    Ok(script)
}

pub(super) fn combat_script_logical_lines(context: &str) -> Vec<String> {
    let mut result = Vec::new();
    for line in context.lines() {
        let line = line
            .trim()
            .replace('（', "(")
            .replace('）', ")")
            .replace('，', ",");
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        if line.contains(';') {
            result.extend(
                line.split(';')
                    .filter_map(|part| {
                        let part = part.trim();
                        (!part.is_empty()).then(|| part.to_string())
                    })
                    .collect::<Vec<_>>(),
            );
        } else {
            result.push(line);
        }
    }
    result
}

pub(super) fn parse_combat_script_lines(
    lines: Vec<String>,
    validate: bool,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<CombatScriptPlan> {
    let mut commands = Vec::new();
    let mut avatar_names = Vec::new();
    for line in lines {
        let mut line_commands = parse_combat_script_line(&line, validate, catalog)?;
        for command in &line_commands {
            if !avatar_names.contains(&command.avatar) {
                avatar_names.push(command.avatar.clone());
            }
        }
        commands.append(&mut line_commands);
    }
    Ok(CombatScriptPlan {
        name: String::new(),
        path: None,
        avatar_names,
        commands,
    })
}

fn parse_combat_script_line(
    line: &str,
    validate: bool,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<Vec<CombatCommandPlan>> {
    let line = line.trim();
    let first_space_index = line.find(' ');
    let (avatar, commands) = match first_space_index {
        Some(index) if index > 0 => {
            let avatar = standardize_combat_script_avatar_name(line[..index].trim(), catalog)?;
            (avatar, line[index + 1..].trim())
        }
        _ if validate => {
            return Err(TaskError::CombatStrategy(
                "combat script line must separate avatar and commands with a space".to_string(),
            ));
        }
        _ => ("当前角色".to_string(), line),
    };
    let mut full_commands = Vec::new();
    for part in commands.split('|').filter(|part| !part.trim().is_empty()) {
        let mut part_commands = parse_combat_script_line_part(part, &avatar)?;
        if part_commands
            .first()
            .map(|command| command.method == CombatCommandMethod::Round)
            .unwrap_or(false)
        {
            let activating_rounds = parse_round_command(&part_commands[0])?;
            part_commands.remove(0);
            for command in &mut part_commands {
                command.activating_rounds = activating_rounds.clone();
            }
        }
        full_commands.extend(part_commands);
    }
    Ok(full_commands)
}

fn standardize_combat_script_avatar_name(
    avatar: &str,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<String> {
    let avatar = avatar.trim();
    if avatar == CURRENT_COMBAT_AVATAR_NAME {
        return Ok(CURRENT_COMBAT_AVATAR_NAME.to_string());
    }
    match catalog {
        Some(catalog) => catalog.standard_name_for_alias(avatar),
        None => Ok(avatar.to_string()),
    }
}

fn parse_combat_script_line_part(part: &str, avatar: &str) -> Result<Vec<CombatCommandPlan>> {
    let command_array: Vec<&str> = part
        .split(',')
        .filter(|command| !command.is_empty())
        .collect();
    let mut commands = Vec::new();
    let mut index = 0;
    while index < command_array.len() {
        let mut command = command_array[index].trim().to_string();
        if command.contains('(') && !command.contains(')') {
            let mut next = index + 1;
            while next < command_array.len() {
                command.push(',');
                command.push_str(command_array[next]);
                if command.matches('(').count() > 1 {
                    return Err(TaskError::CombatStrategy(format!(
                        "combat command has unpaired parentheses: {command}"
                    )));
                }
                if command.contains(')') {
                    index = next;
                    break;
                }
                next += 1;
            }
            if !(command.contains('(') && command.contains(')')) {
                return Err(TaskError::CombatStrategy(format!(
                    "combat command has incomplete parentheses: {command}"
                )));
            }
        }
        commands.push(parse_combat_command(avatar, &command)?);
        index += 1;
    }
    Ok(commands)
}

fn parse_combat_command(avatar: &str, raw_command: &str) -> Result<CombatCommandPlan> {
    let command = raw_command.trim();
    let (method_code, args) = if let Some(start_index) = command.find('(') {
        if start_index == 0 {
            return Err(TaskError::CombatStrategy(format!(
                "combat command is missing method name: {command}"
            )));
        }
        let Some(end_index) = command.find(')') else {
            return Err(TaskError::CombatStrategy(format!(
                "combat command has incomplete parentheses: {command}"
            )));
        };
        let parameters = &command[start_index + 1..end_index];
        (
            command[..start_index].trim(),
            parameters
                .split(',')
                .map(str::trim)
                .map(ToOwned::to_owned)
                .collect(),
        )
    } else {
        (command.trim(), Vec::new())
    };
    let method = CombatCommandMethod::from_code(method_code).ok_or_else(|| {
        TaskError::CombatStrategy(format!("unknown combat strategy method: {method_code}"))
    })?;
    validate_combat_command_args(method, &args)?;
    Ok(CombatCommandPlan {
        avatar: avatar.trim().to_string(),
        method,
        args,
        activating_rounds: Vec::new(),
        raw: command.to_string(),
    })
}

fn parse_round_command(command: &CombatCommandPlan) -> Result<Vec<u32>> {
    if command.args.is_empty() {
        return Err(TaskError::CombatStrategy(
            "round command requires at least one argument".to_string(),
        ));
    }
    let mut rounds = Vec::new();
    for arg in &command.args {
        if let Some((start, end)) = arg.split_once('-') {
            let start = parse_positive_round(start)?;
            let end = parse_positive_round(end)?;
            if start > end {
                return Err(TaskError::CombatStrategy(
                    "round range start must be less than or equal to end".to_string(),
                ));
            }
            rounds.extend(start..=end);
        } else {
            rounds.push(parse_positive_round(arg)?);
        }
    }
    Ok(rounds)
}

fn parse_positive_round(value: &str) -> Result<u32> {
    let round: u32 = value
        .trim()
        .parse()
        .map_err(|_| TaskError::CombatStrategy(format!("invalid round value: {value}")))?;
    if round == 0 {
        return Err(TaskError::CombatStrategy(
            "round value must be greater than zero".to_string(),
        ));
    }
    Ok(round)
}

fn validate_combat_command_args(method: CombatCommandMethod, args: &[String]) -> Result<()> {
    match method {
        CombatCommandMethod::Walk => {
            if args.len() != 2 {
                return Err(TaskError::CombatStrategy(
                    "walk command requires direction and duration".to_string(),
                ));
            }
            parse_positive_f64(&args[1], "walk duration")?;
        }
        CombatCommandMethod::W
        | CombatCommandMethod::A
        | CombatCommandMethod::S
        | CombatCommandMethod::D => {
            if args.len() != 1 {
                return Err(TaskError::CombatStrategy(
                    "w/a/s/d command requires one duration argument".to_string(),
                ));
            }
            parse_f64_arg(&args[0], "walk duration")?;
        }
        CombatCommandMethod::MoveBy => {
            if args.len() != 2 {
                return Err(TaskError::CombatStrategy(
                    "moveby command requires x and y arguments".to_string(),
                ));
            }
            parse_i32_arg(&args[0], "moveby x")?;
            parse_i32_arg(&args[1], "moveby y")?;
        }
        CombatCommandMethod::KeyDown
        | CombatCommandMethod::KeyUp
        | CombatCommandMethod::KeyPress => {
            if args.len() != 1 {
                return Err(TaskError::CombatStrategy(
                    "key command requires one key argument".to_string(),
                ));
            }
            validate_virtual_key_name(&args[0])?;
        }
        CombatCommandMethod::Scroll => {
            if args.len() != 1 {
                return Err(TaskError::CombatStrategy(
                    "scroll command requires one integer argument".to_string(),
                ));
            }
            parse_i32_arg(&args[0], "scroll amount")?;
        }
        _ => {}
    }
    Ok(())
}

fn parse_positive_f64(value: &str, label: &str) -> Result<f64> {
    let parsed = parse_f64_arg(value, label)?;
    if parsed <= 0.0 {
        return Err(TaskError::CombatStrategy(format!(
            "{label} must be positive"
        )));
    }
    Ok(parsed)
}

pub(super) fn parse_f64_arg(value: &str, label: &str) -> Result<f64> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| TaskError::CombatStrategy(format!("{label} must be a number: {value}")))
}

pub(super) fn parse_i32_arg(value: &str, label: &str) -> Result<i32> {
    value
        .trim()
        .parse::<i32>()
        .map_err(|_| TaskError::CombatStrategy(format!("{label} must be an integer: {value}")))
}

fn validate_virtual_key_name(value: &str) -> Result<()> {
    combat_virtual_key_plan(value).map(|_| ())
}

pub(super) fn collect_txt_files(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_txt_files(&path, files)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("txt"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
    Ok(())
}

pub(super) fn normalize_user_auto_fight_strategy_path(strategy_path: &str) -> Result<PathBuf> {
    let strategy_path = strategy_path.trim().replace('\\', "/");
    if strategy_path.is_empty() {
        return Err(TaskError::EmptyCombatStrategyPath);
    }
    let path = PathBuf::from(&strategy_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(TaskError::InvalidCombatStrategyPath(strategy_path));
    }
    if !strategy_path
        .split('/')
        .next()
        .map(|first| first.eq_ignore_ascii_case("User"))
        .unwrap_or(false)
    {
        return Err(TaskError::InvalidCombatStrategyPath(strategy_path));
    }
    let mut components = path.components();
    let _ = components.next();
    let Some(second) = components.next() else {
        return Err(TaskError::InvalidCombatStrategyPath(strategy_path));
    };
    if second.as_os_str() != "AutoFight" {
        return Err(TaskError::InvalidCombatStrategyPath(strategy_path));
    }
    Ok(path)
}
