#[derive(Debug, serde::Deserialize)]
pub struct TrackMetadata {
    #[serde(rename = "#")]
    pub id: u32,

    #[serde(rename = "Track Title")]
    pub title: String,

    #[serde(rename = "Artist")]
    pub artist: String,

    #[serde(rename = "BPM")]
    pub bpm: String,

    #[serde(rename = "Key")]
    pub key: String,

    #[serde(rename = "Time")]
    pub time: String,

    #[serde(rename = "Message")]
    pub message: String,

    #[serde(rename = "Date Added")]
    date_added: String,

    #[serde(rename = "Genre")]
    genre: String,

    #[serde(rename = "My Tag")]
    pub my_tag: String,

    #[serde(skip_deserializing)]
    pub media_segment: m3u8_rs::MediaSegment,
}

