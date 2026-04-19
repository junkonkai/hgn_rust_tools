mod hash;
mod image;

use serde::{Deserialize, Serialize};
use std::process;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Review(ReviewPayload),
}

#[derive(Debug, Deserialize)]
pub struct ReviewPayload {
    pub review_id: i64,
    pub game_title_name: String,
    pub user_name: String,
    pub total_score: Option<i64>,
    pub fear_meter: Option<i64>,
    pub score_story: Option<i64>,
    pub score_atmosphere: Option<i64>,
    pub score_gameplay: Option<i64>,
    pub user_score_adjustment: Option<i64>,
    pub has_spoiler: bool,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
    Ok { ok: bool, filename: String },
    Err { ok: bool, error: String },
}

fn run() -> Result<String, String> {
    // 第1引数からJSONを取得
    let json_arg = std::env::args()
        .nth(1)
        .ok_or_else(|| "Usage: ogp-generator '<JSON>'".to_string())?;

    // JSONをパース（"type"フィールドでディスパッチ）
    let payload: Payload = serde_json::from_str(&json_arg)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // 環境変数を読み込む
    let output_dir = std::env::var("OUTPUT_DIR")
        .map_err(|_| "Environment variable OUTPUT_DIR is not set".to_string())?;
    let font_path = std::env::var("FONT_PATH")
        .map_err(|_| "Environment variable FONT_PATH is not set".to_string())?;
    let template_path = std::env::var("SVG_TEMPLATE_PATH")
        .map_err(|_| "Environment variable SVG_TEMPLATE_PATH is not set".to_string())?;

    // SVGテンプレートを読み込む
    let template = std::fs::read_to_string(&template_path)
        .map_err(|e| format!("Failed to read SVG template '{}': {}", template_path, e))?;

    match payload {
        Payload::Review(review) => {
            let png_bytes = image::render(&review, &template, &font_path)
                .map_err(|e| format!("Failed to render image: {}", e))?;
            let filename = hash::review_id_to_filename(review.review_id);
            let output_path = std::path::Path::new(&output_dir).join(&filename);
            std::fs::write(&output_path, &png_bytes)
                .map_err(|e| format!("Failed to write PNG '{}': {}", output_path.display(), e))?;
            Ok(filename)
        }
    }
}

fn main() {
    match run() {
        Ok(filename) => {
            let response = Response::Ok { ok: true, filename };
            println!("{}", serde_json::to_string(&response).unwrap());
            process::exit(0);
        }
        Err(error) => {
            let response = Response::Err { ok: false, error };
            println!("{}", serde_json::to_string(&response).unwrap());
            process::exit(1);
        }
    }
}
