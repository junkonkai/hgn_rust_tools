use tiny_skia::Pixmap;
use usvg::Tree;

use crate::Payload;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// SVGテンプレートにペイロードを埋め込んでPNGバイト列を返す
pub fn render(payload: &Payload, template: &str, font_path: &str) -> Result<Vec<u8>> {
    let svg = build_svg(payload, template);

    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_font_file(font_path)?;

    let tree = Tree::from_str(&svg, &opt)?;

    let mut pixmap = Pixmap::new(1200, 630).ok_or("Failed to allocate pixmap")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    Ok(pixmap.encode_png()?)
}

/// プレースホルダーを置換してSVG文字列を生成する
fn build_svg(payload: &Payload, template: &str) -> String {
    let spoiler_badge = if payload.has_spoiler {
        "<rect x=\"40\" y=\"30\" width=\"280\" height=\"56\" rx=\"8\" fill=\"#dc2626\"/>\
         <text x=\"180\" y=\"67\" font-size=\"30\" fill=\"#ffffff\" text-anchor=\"middle\" \
         font-family=\"NotoSansJP, Noto Sans CJK JP, sans-serif\">【ネタバレあり】</text>"
    } else {
        ""
    };

    let title_font_size = if payload.game_title_name.chars().count() > 30 {
        "44"
    } else {
        "64"
    };

    let total_score = match payload.total_score {
        Some(s) => s.to_string(),
        None => "-".to_string(),
    };

    let axis_scores = build_axis_scores(payload);

    template
        .replace("{{SPOILER_BADGE}}", spoiler_badge)
        .replace("{{GAME_TITLE}}", &escape_xml(&payload.game_title_name))
        .replace("{{TITLE_FONT_SIZE}}", title_font_size)
        .replace("{{SHOW_ID}}", &escape_xml(&payload.show_id))
        .replace("{{TOTAL_SCORE}}", &total_score)
        .replace("{{AXIS_SCORES}}", &escape_xml(&axis_scores))
}

/// null でない軸スコアのみを全角スペース区切りで結合する
fn build_axis_scores(payload: &Payload) -> String {
    let mut parts = Vec::new();
    if let Some(v) = payload.fear_meter {
        parts.push(format!("怖さ {}/4", v));
    }
    if let Some(v) = payload.score_story {
        parts.push(format!("ストーリー {}/4", v));
    }
    if let Some(v) = payload.score_atmosphere {
        parts.push(format!("雰囲気 {}/4", v));
    }
    if let Some(v) = payload.score_gameplay {
        parts.push(format!("ゲーム性 {}/4", v));
    }
    parts.join("　")
}

/// SVGテキストコンテンツ用のXMLエスケープ
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Payload;

    const TEMPLATE: &str = include_str!("../assets/review-template.svg");

    fn sample_payload() -> Payload {
        Payload {
            review_id: 1,
            game_title_name: "バイオハザード RE:2".to_string(),
            show_id: "huckle".to_string(),
            total_score: Some(95),
            fear_meter: Some(3),
            score_story: Some(4),
            score_atmosphere: Some(4),
            score_gameplay: Some(3),
            has_spoiler: false,
        }
    }

    fn find_font() -> Option<String> {
        if let Ok(p) = std::env::var("FONT_PATH") {
            if std::path::Path::new(&p).exists() {
                return Some(p);
            }
        }
        let candidates = [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Bold.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ];
        candidates
            .iter()
            .find(|p| std::path::Path::new(p).exists())
            .map(|p| p.to_string())
    }

    #[test]
    fn test_render_returns_png_bytes() {
        let Some(font) = find_font() else {
            eprintln!("SKIP: no font found; set FONT_PATH to run this test");
            return;
        };
        let payload = sample_payload();
        let result = render(&payload, TEMPLATE, &font);
        assert!(result.is_ok(), "render failed: {:?}", result.err());

        let png = result.unwrap();
        // PNGマジックバイト確認
        assert_eq!(
            &png[..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            "output is not a valid PNG"
        );
        // 目視確認用に書き出す
        std::fs::write("/tmp/ogp_test.png", &png).unwrap();
        eprintln!("PNG written to /tmp/ogp_test.png for visual inspection");
    }

    #[test]
    fn test_build_axis_scores_all_present() {
        let payload = sample_payload();
        let result = build_axis_scores(&payload);
        assert!(result.contains("怖さ 3/4"));
        assert!(result.contains("ストーリー 4/4"));
        assert!(result.contains("雰囲気 4/4"));
        assert!(result.contains("ゲーム性 3/4"));
    }

    #[test]
    fn test_build_axis_scores_partial_null() {
        let payload = Payload {
            fear_meter: None,
            score_story: Some(2),
            score_atmosphere: None,
            score_gameplay: Some(4),
            ..sample_payload()
        };
        let result = build_axis_scores(&payload);
        assert!(!result.contains("怖さ"));
        assert!(result.contains("ストーリー 2/4"));
        assert!(!result.contains("雰囲気"));
        assert!(result.contains("ゲーム性 4/4"));
    }

    #[test]
    fn test_build_svg_null_total_score() {
        let payload = Payload {
            total_score: None,
            ..sample_payload()
        };
        let svg = build_svg(&payload, TEMPLATE);
        assert!(svg.contains("-点"), "null total_score should render as '-点'");
    }

    #[test]
    fn test_build_svg_spoiler_badge() {
        let with_spoiler = Payload {
            has_spoiler: true,
            ..sample_payload()
        };
        let without_spoiler = sample_payload();

        let svg_with = build_svg(&with_spoiler, TEMPLATE);
        let svg_without = build_svg(&without_spoiler, TEMPLATE);

        assert!(
            svg_with.contains("【ネタバレあり】"),
            "has_spoiler=true should include badge"
        );
        assert!(
            !svg_without.contains("【ネタバレあり】"),
            "has_spoiler=false should not include badge"
        );
    }

    #[test]
    fn test_build_svg_long_title_font_size() {
        let long_title = Payload {
            game_title_name: "あ".repeat(31),
            ..sample_payload()
        };
        let short_title = sample_payload();

        let svg_long = build_svg(&long_title, TEMPLATE);
        let svg_short = build_svg(&short_title, TEMPLATE);

        assert!(svg_long.contains("font-size=\"44\""));
        assert!(svg_short.contains("font-size=\"64\""));
    }

    #[test]
    fn test_escape_xml_special_chars() {
        assert_eq!(escape_xml("A&B"), "A&amp;B");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("say \"hi\""), "say &quot;hi&quot;");
    }
}
