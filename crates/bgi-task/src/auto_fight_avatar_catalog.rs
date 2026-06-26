use crate::{task_asset_root, Result, TaskError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const COMBAT_AVATAR_CATALOG_PATH: &str = "GameTask/AutoFight/Assets/combat_avatar.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatAvatarMetadata {
    #[serde(default)]
    pub alias: Vec<String>,
    pub id: String,
    pub name: String,
    #[serde(rename = "nameEn")]
    pub name_en: String,
    pub weapon: String,
    #[serde(default, rename = "skillCD")]
    pub skill_cd: Option<f64>,
    #[serde(default, rename = "skillHoldCD")]
    pub skill_hold_cd: Option<f64>,
    #[serde(default, rename = "burstCD")]
    pub burst_cd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CombatAvatarCatalog {
    pub source_path: PathBuf,
    pub avatars: Vec<CombatAvatarMetadata>,
    pub alias_to_name: BTreeMap<String, String>,
}

impl CombatAvatarCatalog {
    pub fn new(source_path: PathBuf, avatars: Vec<CombatAvatarMetadata>) -> Result<Self> {
        let mut alias_to_name = BTreeMap::new();
        for avatar in &avatars {
            for alias in &avatar.alias {
                let alias = alias.trim();
                if alias.is_empty() {
                    continue;
                }
                if let Some(existing) = alias_to_name.insert(alias.to_string(), avatar.name.clone())
                {
                    return Err(TaskError::CombatStrategy(format!(
                        "duplicate combat avatar alias: {alias} maps to both {existing} and {}",
                        avatar.name
                    )));
                }
            }
        }
        Ok(Self {
            source_path,
            avatars,
            alias_to_name,
        })
    }

    pub fn standard_name_for_alias(&self, alias: &str) -> Result<String> {
        let alias = alias.trim();
        self.alias_to_name
            .get(alias)
            .cloned()
            .ok_or_else(|| TaskError::CombatStrategy(format!("角色名称校验失败：{alias}")))
    }

    pub fn avatar_by_name(&self, name: &str) -> Option<&CombatAvatarMetadata> {
        self.avatars.iter().find(|avatar| avatar.name == name)
    }

    pub fn avatar_by_name_en(&self, name_en: &str) -> Option<&CombatAvatarMetadata> {
        self.avatars.iter().find(|avatar| avatar.name_en == name_en)
    }
}

pub fn resolve_combat_avatar_catalog_path(working_directory: impl AsRef<Path>) -> PathBuf {
    let root = working_directory.as_ref();
    [
        root.join(COMBAT_AVATAR_CATALOG_PATH),
        task_asset_root().join(COMBAT_AVATAR_CATALOG_PATH),
    ]
    .into_iter()
    .find(|path| path.exists())
    .unwrap_or_else(|| root.join(COMBAT_AVATAR_CATALOG_PATH))
}

pub fn read_combat_avatar_catalog(
    working_directory: impl AsRef<Path>,
) -> Result<CombatAvatarCatalog> {
    let path = resolve_combat_avatar_catalog_path(working_directory);
    let json = fs::read_to_string(&path).map_err(|error| {
        TaskError::CombatStrategy(format!(
            "combat avatar catalog cannot be read: {} ({error})",
            path.display()
        ))
    })?;
    let avatars: Vec<CombatAvatarMetadata> = serde_json::from_str(&json).map_err(|error| {
        TaskError::CombatStrategy(format!(
            "combat avatar catalog failed to parse: {} ({error})",
            path.display()
        ))
    })?;
    CombatAvatarCatalog::new(path, avatars)
}
