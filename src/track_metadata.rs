#[derive(Debug, serde::Deserialize, Clone)]
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

pub enum KeyCompareResult {
    PerfectMatch,
    Compatible,
    MaybeCompatible,
    Incopmatible,
}

pub fn color_from_key_compare(key: KeyCompareResult) -> Option<eframe::egui::Color32> {
    match key {
        KeyCompareResult::PerfectMatch => Some(eframe::egui::Color32::from_rgb(11, 75, 128)),
        KeyCompareResult::Compatible => Some(eframe::egui::Color32::from_rgb(19, 128, 11)),
        KeyCompareResult::MaybeCompatible => Some(eframe::egui::Color32::from_rgb(143, 119, 11)),
        KeyCompareResult::Incopmatible => None,
    }
}

/** 
* Compare keys in camelot format
*/
pub fn compare_keys(a: &str, b: &str) -> Result<KeyCompareResult, &'static str> {
    if a == b {
        return Ok(KeyCompareResult::PerfectMatch);
    }

    let (a_key, a_mode) = a.split_at(a.len() - 1);
    let (b_key, b_mode) = b.split_at(b.len() - 1);

    let key_distance = (a_key.parse::<i32>().unwrap() - b_key.parse::<i32>().unwrap()).abs();

    match key_distance {
        0 => Ok(KeyCompareResult::Compatible),
        1 | 2 => {
            if a_mode == b_mode {
                Ok(KeyCompareResult::Compatible)
            } else {
                Ok(KeyCompareResult::MaybeCompatible)
            }
        },
        3 | 5 | 7 => {
            if a_mode == b_mode {
                Ok(KeyCompareResult::MaybeCompatible)
            } else {
                Ok(KeyCompareResult::Incopmatible)
            }
        },
        _ => Ok(KeyCompareResult::Incopmatible)
    }
}

