use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorScript {
    pub id: String,
    pub duration: Option<f32>,
    pub keyframes: Vec<Keyframe>,
    pub next: Option<String>,
    pub interruptible: bool,
}

impl BehaviorScript {
    /// Load a behavior script from a RON file
    pub fn load_from_ron<P: AsRef<Path>>(path: P) -> Result<Self, ScriptError> {
        let content = fs::read_to_string(&path).map_err(|e| ScriptError::Io {
            path: path.as_ref().display().to_string(),
            source: e,
        })?;
        Self::parse_ron(&content)
    }

    /// Parse a behavior script from a RON string
    pub fn parse_ron(content: &str) -> Result<Self, ScriptError> {
        ron::from_str(content).map_err(ScriptError::Parse)
    }

    /// Serialize the script to RON format
    pub fn to_ron(&self) -> Result<String, ScriptError> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(ScriptError::Serialize)
    }
}

/// Errors that can occur when loading or saving scripts
#[derive(Debug)]
pub enum ScriptError {
    Io {
        path: String,
        source: std::io::Error,
    },
    Parse(ron::error::SpannedError),
    Serialize(ron::Error),
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "failed to read script file '{}': {}", path, source)
            }
            Self::Parse(e) => write!(f, "failed to parse RON script: {}", e),
            Self::Serialize(e) => write!(f, "failed to serialize script to RON: {}", e),
        }
    }
}

impl std::error::Error for ScriptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Parse(e) => Some(e),
            Self::Serialize(e) => Some(e),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    PlayAnimation { name: String },
    MoveTo { x: f32, y: f32 },
    Scale { factor: f32 },
    SetExpression { expression: String },
    Wait { duration: f32 },
    Random { scripts: Vec<String> },
}

impl BehaviorScript {
    #[must_use]
    pub fn idle() -> Self {
        Self {
            id: "idle".to_string(),
            duration: Some(3.0),
            keyframes: vec![
                Keyframe {
                    time: 0.0,
                    action: Action::PlayAnimation {
                        name: "idle".to_string(),
                    },
                },
                Keyframe {
                    time: 0.0,
                    action: Action::MoveTo { x: 0.0, y: 0.0 },
                },
                Keyframe {
                    time: 3.0,
                    action: Action::Random {
                        scripts: vec!["idle".to_string(), "idle".to_string(), "walk".to_string()],
                    },
                },
            ],
            next: None,
            interruptible: true,
        }
    }

    #[must_use]
    pub fn walk() -> Self {
        Self {
            id: "walk".to_string(),
            duration: Some(4.0),
            keyframes: vec![
                Keyframe {
                    time: 0.0,
                    action: Action::PlayAnimation {
                        name: "walk".to_string(),
                    },
                },
                Keyframe {
                    time: 0.0,
                    action: Action::MoveTo { x: 0.3, y: 0.0 },
                },
                Keyframe {
                    time: 2.0,
                    action: Action::MoveTo { x: -0.3, y: 0.0 },
                },
            ],
            next: Some("idle".to_string()),
            interruptible: true,
        }
    }

    #[must_use]
    pub fn happy() -> Self {
        Self {
            id: "happy".to_string(),
            duration: Some(2.0),
            keyframes: vec![
                Keyframe {
                    time: 0.0,
                    action: Action::PlayAnimation {
                        name: "jump".to_string(),
                    },
                },
                Keyframe {
                    time: 0.5,
                    action: Action::Scale { factor: 1.2 },
                },
                Keyframe {
                    time: 1.0,
                    action: Action::Scale { factor: 1.0 },
                },
                Keyframe {
                    time: 1.5,
                    action: Action::PlayAnimation {
                        name: "spin".to_string(),
                    },
                },
            ],
            next: Some("idle".to_string()),
            interruptible: false,
        }
    }

    #[must_use]
    pub fn sleep() -> Self {
        Self {
            id: "sleep".to_string(),
            duration: None,
            keyframes: vec![Keyframe {
                time: 0.0,
                action: Action::PlayAnimation {
                    name: "sleep".to_string(),
                },
            }],
            next: None,
            interruptible: true,
        }
    }

    #[must_use]
    pub fn builtin_scripts() -> Vec<Self> {
        vec![Self::idle(), Self::walk(), Self::happy(), Self::sleep()]
    }
}
