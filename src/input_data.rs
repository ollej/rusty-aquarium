use crate::{fish_data::FishData, legend::Legend};
use macroquad::{file::load_string, prelude::error, window::next_frame};
use nanoserde::DeJson;
use quad_net::http_request::RequestBuilder;

#[derive(Clone, Default, DeJson)]
pub struct InputData {
    pub legend: Option<Legend>,
    pub school: Vec<FishData>,
}

impl InputData {
    pub async fn load(path: String) -> Self {
        let json = if Self::is_url(&path) {
            Self::load_url(path).await
        } else {
            load_string(path.as_str()).await.ok()
        };
        DeJson::deserialize_json(&json.unwrap_or_else(|| "{}".to_string())).unwrap_or_default()
    }

    fn is_url(path: &str) -> bool {
        path.starts_with("http://") || path.starts_with("https://")
    }

    async fn load_url(path: String) -> Option<String> {
        let mut request = RequestBuilder::new(path.as_str()).send();
        loop {
            if let Some(result) = request.try_recv() {
                return match result {
                    Ok(data) => Some(data),
                    Err(error) => {
                        error!("Error reading inputdata: {}", error);
                        None
                    }
                };
            }
            next_frame().await;
        }
    }
}
