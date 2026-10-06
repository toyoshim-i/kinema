use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Failed to read project file '{0}': {1}")]
    Io(PathBuf, std::io::Error),
    #[error("Failed to parse TOML in '{0}': {1}")]
    Toml(PathBuf, Box<toml::de::Error>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSection {
    pub name: Option<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub libraries: Vec<String>,
    pub board: Option<String>,
}

impl Default for ProjectSection {
    fn default() -> Self {
        Self {
            name: Some("kinema_project".into()),
            sources: vec!["src/*.v".into()],
            libraries: vec!["lib/*.v".into()],
            board: Some("board.kicad_pcb".into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KicadSection {
    pub config_dir: Option<PathBuf>,
    pub footprint_dir: Option<PathBuf>,
    pub fp_lib_table: Option<PathBuf>,
    #[serde(default)]
    pub search_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct ProjectConfig {
    pub project: ProjectSection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kicad: Option<KicadSection>,
}

#[derive(Deserialize)]
struct RawProjectConfig {
    #[serde(default)]
    project: Option<ProjectSection>,
    #[serde(default)]
    kicad: Option<KicadSection>,
    name: Option<String>,
    #[serde(default)]
    sources: Option<Vec<String>>,
    #[serde(default)]
    libraries: Option<Vec<String>>,
    board: Option<String>,
}

impl<'de> Deserialize<'de> for ProjectConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawProjectConfig::deserialize(deserializer)?;
        let project = match raw.project {
            Some(mut sec) => {
                if sec.name.is_none() {
                    sec.name = raw.name;
                }
                if sec.board.is_none() {
                    sec.board = raw.board;
                }
                sec
            }
            None => {
                let default = ProjectSection::default();
                ProjectSection {
                    name: raw.name.or(default.name),
                    sources: raw.sources.unwrap_or(default.sources),
                    libraries: raw.libraries.unwrap_or(default.libraries),
                    board: raw.board.or(default.board),
                }
            }
        };
        Ok(ProjectConfig {
            project,
            kicad: raw.kicad,
        })
    }
}

impl ProjectConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| ProjectError::Io(path.to_path_buf(), e))?;
        toml::from_str(&content)
            .map_err(|e| ProjectError::Toml(path.to_path_buf(), Box::new(e)))
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        Self::load_from_file(path).unwrap_or_default()
    }
}
