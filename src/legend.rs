use crate::fish_legend::FishLegend;
use nanoserde::{DeJson, SerJson};

#[derive(Clone, Debug, DeJson, SerJson)]
pub struct Legend {
    pub description: String,
    pub fish_legends: Vec<FishLegend>,
}
