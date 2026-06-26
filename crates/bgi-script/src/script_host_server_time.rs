use super::{Result, ScriptHostRuntimeError};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ServerTimeHost {
    server_time_zone_offset_milliseconds: i32,
}

impl ServerTimeHost {
    pub fn from_offset_milliseconds(server_time_zone_offset_milliseconds: i32) -> Self {
        Self {
            server_time_zone_offset_milliseconds,
        }
    }

    pub fn from_offset_string(offset: &str) -> Result<Self> {
        Ok(Self::from_offset_milliseconds(
            parse_server_time_zone_offset_milliseconds(offset)?,
        ))
    }

    pub fn server_time_zone_offset_milliseconds(&self) -> i32 {
        self.server_time_zone_offset_milliseconds
    }
}

impl Default for ServerTimeHost {
    fn default() -> Self {
        Self::from_offset_milliseconds(8 * 60 * 60 * 1_000)
    }
}

fn parse_server_time_zone_offset_milliseconds(offset: &str) -> Result<i32> {
    let trimmed = offset.trim();
    if trimmed.is_empty() {
        return Err(ScriptHostRuntimeError::InvalidServerTimeZoneOffset(
            offset.to_string(),
        ));
    }

    let (sign, value) = match trimmed.as_bytes()[0] {
        b'-' => (-1_i64, &trimmed[1..]),
        b'+' => (1_i64, &trimmed[1..]),
        _ => (1_i64, trimmed),
    };

    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(ScriptHostRuntimeError::InvalidServerTimeZoneOffset(
            offset.to_string(),
        ));
    }

    let hours = parse_offset_component(parts[0], offset)?;
    let minutes = parse_offset_component(parts[1], offset)?;
    let seconds = parse_offset_component(parts[2], offset)?;
    if minutes >= 60 || seconds >= 60 {
        return Err(ScriptHostRuntimeError::InvalidServerTimeZoneOffset(
            offset.to_string(),
        ));
    }

    let milliseconds =
        sign * ((hours * 60 * 60 * 1_000) + (minutes * 60 * 1_000) + (seconds * 1_000));
    i32::try_from(milliseconds)
        .map_err(|_| ScriptHostRuntimeError::InvalidServerTimeZoneOffset(offset.to_string()))
}

fn parse_offset_component(component: &str, original: &str) -> Result<i64> {
    component
        .parse::<i64>()
        .map_err(|_| ScriptHostRuntimeError::InvalidServerTimeZoneOffset(original.to_string()))
}
