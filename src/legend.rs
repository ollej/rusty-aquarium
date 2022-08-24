use crate::fish_legend::FishLegend;
use nanoserde::DeJson;

#[derive(Clone, Debug, DeJson)]
pub struct Legend {
    pub description: String,
    pub fish_legends: Vec<FishLegend>,
}
