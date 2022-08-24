#![allow(clippy::question_mark)]

use crate::legend::Legend;
use nanoserde::DeJson;

#[derive(Clone, Debug, DeJson)]
#[nserde(default)]
pub struct SceneConfig {
    pub input_data_path: Option<String>,
    pub display_time: u32,
    pub background: Option<usize>,
    pub legend: Option<Legend>,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            input_data_path: None,
            display_time: 30,
            background: None,
            legend: None,
        }
    }
}

impl SceneConfig {
    pub fn new(input_data_path: Option<String>, display_time: u32) -> Self {
        Self {
            input_data_path,
            display_time,
            background: None,
            legend: None,
        }
    }
}
