use nanoserde::DeJson;

#[derive(Clone, DeJson, Debug)]
pub struct FishLegend {
    pub fish: String,
    pub description: String,
}
