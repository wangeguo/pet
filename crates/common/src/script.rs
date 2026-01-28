use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorScript {
    pub id: String,
    pub duration: Option<f32>,
    pub keyframes: Vec<Keyframe>,
    pub next: Option<String>,
    pub interruptible: bool,
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
            duration: None,
            keyframes: vec![
                Keyframe {
                    time: 0.0,
                    action: Action::PlayAnimation {
                        name: "breathe".to_string(),
                    },
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
                    action: Action::MoveTo { x: 100.0, y: 0.0 },
                },
                Keyframe {
                    time: 2.0,
                    action: Action::MoveTo { x: -100.0, y: 0.0 },
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
