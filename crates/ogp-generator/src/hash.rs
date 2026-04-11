use sha2::{Digest, Sha256};

pub fn review_id_to_filename(review_id: i64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(review_id.to_string().as_bytes());
    let result = hasher.finalize();
    format!("{}.png", hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_id_1() {
        assert_eq!(
            review_id_to_filename(1),
            "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b.png"
        );
    }

    #[test]
    fn test_review_id_42() {
        assert_eq!(
            review_id_to_filename(42),
            "73475cb40a568e8da8a045ced110137e159f890ac4da883b6b17dc651b3a8049.png"
        );
    }

    #[test]
    fn test_output_format() {
        let filename = review_id_to_filename(1);
        assert!(filename.ends_with(".png"));
        // SHA-256 hex は 64文字
        assert_eq!(filename.len(), 64 + 4); // hex(64) + ".png"(4)
    }
}
