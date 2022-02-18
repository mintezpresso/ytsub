use serde_json::Value;

pub enum RefreshState {
    ToBeRefreshed,
    Refreshing,
    Completed,
}

pub struct Channel {
    pub channel_id: String,
    pub channel_name: String,
    pub refresh_state: RefreshState,
    pub new_video: bool,
}

impl Channel {
    pub fn new(channel_id: String, channel_name: String) -> Self {
        Self {
            channel_id,
            channel_name,
            refresh_state: RefreshState::Completed,
            new_video: false,
        }
    }

    pub fn set_to_be_refreshed(&mut self) {
        self.refresh_state = RefreshState::ToBeRefreshed;
    }
}

pub struct Video {
    pub video_id: String,
    pub title: String,
    pub published: u32,
    pub length: u32,
    pub watched: bool,
    pub new: bool,
}

impl Video {
    pub fn from_json(video_json: &Value) -> Self {
        Video {
            video_id: video_json
                .get("videoId")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            title: video_json
                .get("title")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            published: video_json.get("published").unwrap().as_u64().unwrap() as u32,
            length: video_json.get("lengthSeconds").unwrap().as_u64().unwrap() as u32,
            watched: false,
            new: true,
        }
    }

    pub fn vec_from_json(videos_json: Value) -> Vec<Video> {
        videos_json
            .as_array()
            .unwrap()
            .iter()
            .map(Video::from_json)
            .collect()
    }
}
