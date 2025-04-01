#![allow(clippy::redundant_closure_for_method_calls)]

use super::*;
use lazy_static::lazy_static;
use serde_json::{json, Value};

pub trait ToJson {
    fn as_json(&self) -> Value;
}

const EME_MESSAGE_SCHEMA: &str = include_str!(
    "../../../eme/assets/json/schema/eme/realtime_request_schema.json"
);

lazy_static! {
    static ref EME_MSG_SCHEMA_VALIDATOR: jsonschema::Validator = {
        let schema: Value = serde_json::from_str(EME_MESSAGE_SCHEMA)
            .expect("failed to parse EME message schema to JSON");
        jsonschema::Validator::new(&schema)
            .expect("failed to create EME message schema validator")
    };
}

fn is_eme_msg_valid(json: &Value) -> bool {
    EME_MSG_SCHEMA_VALIDATOR.is_valid(json)
}

// *** *** *** //

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EMEPlayback {
    Start,
    Stop,
}

impl ToJson for EMEPlayback {
    fn as_json(&self) -> Value {
        match self {
            Self::Start => json!("start"),
            Self::Stop => json!("stop"),
        }
    }
}

// *** *** *** //

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EMEPosition {
    pub x: f32,
    pub y: f32,
}

impl EMEPosition {
    #[rustfmt::skip]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x.clamp(-1.0, 1.0),
            y: y.clamp(0.0, 1.0),
        }
    }
}

impl ToJson for EMEPosition {
    fn as_json(&self) -> Value {
        json!({
            "x": json!(self.x),
            "y": json!(self.y),
        })
    }
}

// *** *** *** //

#[derive(Clone, Debug, PartialEq)]
pub struct EMERequest {
    pub arrangement: Option<String>,
    pub playback: Option<EMEPlayback>,
    pub position: Option<EMEPosition>,
}

impl EMERequest {
    pub const fn new() -> Self {
        Self { arrangement: None, playback: None, position: None }
    }

    pub const fn with_playback(mut self, playback: EMEPlayback) -> Self {
        self.playback = Some(playback);
        self
    }

    pub const fn with_position(mut self, position: EMEPosition) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_arrangement(mut self, arrangement: &str) -> Self {
        self.arrangement = Some(String::from(arrangement));
        self
    }

    pub fn is_start(&self) -> bool {
        self.playback.is_some_and(|p| p == EMEPlayback::Start)
    }

    pub fn is_stop(&self) -> bool {
        self.playback.is_some_and(|p| p == EMEPlayback::Stop)
    }
}

impl Default for EMERequest {
    fn default() -> Self {
        Self::new()
    }
}

impl ToJson for EMERequest {
    fn as_json(&self) -> Value {
        let mut result = json!({});

        let obj = unsafe { result.as_object_mut().unwrap_unchecked() };

        if let Some(arrangement) = &self.arrangement {
            let json = json!(arrangement);
            obj.insert("arrangement".into(), json);
        }

        if let Some(playback) = &self.playback {
            let json = playback.as_json();
            obj.insert("playback".into(), json);
        }

        if let Some(pos) = &self.position {
            let json = pos.as_json();
            obj.insert("position".into(), json);
        }

        assert!(
            is_eme_msg_valid(&result),
            "failed to validate EME message: {result}"
        );

        // println!("for {self:?}, got \"{result}\"");

        result
    }
}
