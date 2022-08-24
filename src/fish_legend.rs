use nanoserde::{DeJson, SerJson};

#[derive(Clone, Debug, DeJson, SerJson)]
pub struct FishLegend {
    pub fish: String,
    pub description: String,
}
