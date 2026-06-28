use serde::{Deserialize, Serialize};

pub const TEYVAT_256_SIFT_KEYPOINTS: &str = "Assets/Map/Teyvat/Teyvat_0_256_SIFT.kp.bin";
pub const TEYVAT_256_SIFT_MAT: &str = "Assets/Map/Teyvat/Teyvat_0_256_SIFT.mat.png";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BigMapSiftRecognitionRule {
    pub matching_method: String,
    pub feature_detector: String,
    pub matcher: String,
    pub descriptor_format: String,
    pub query_downscale: f64,
    pub feature_layer: BigMapSiftFeatureLayerRule,
    pub feature_keypoints_asset: String,
    pub feature_mat_asset: String,
    pub feature_split_rows: u64,
    pub feature_split_cols: u64,
    pub prev_rect_expand_blocks: u64,
    pub fallback_full_search: bool,
    pub center_match_rule: BigMapSiftMatchRule,
    pub rect_match_rule: BigMapSiftMatchRule,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BigMapSiftFeatureLayerRule {
    pub map: String,
    pub floor: i32,
    pub source_tile_size: u64,
    pub image_to_2048_scale: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BigMapSiftMatchRule {
    pub operation: String,
    pub ratio: f64,
    pub min_good_matches: u64,
    pub ransac_reprojection_threshold: f64,
}

pub fn teyvat_big_map_sift_recognition_rule() -> BigMapSiftRecognitionRule {
    BigMapSiftRecognitionRule {
        matching_method: "SIFT".to_string(),
        feature_detector: "SIFT".to_string(),
        matcher: "FlannBased".to_string(),
        descriptor_format: "OpenCvSharp KeyPoint raw bin + grayscale png converted to CV_32FC1"
            .to_string(),
        query_downscale: 0.25,
        feature_layer: BigMapSiftFeatureLayerRule {
            map: "Teyvat".to_string(),
            floor: 0,
            source_tile_size: 256,
            image_to_2048_scale: 8,
        },
        feature_keypoints_asset: TEYVAT_256_SIFT_KEYPOINTS.to_string(),
        feature_mat_asset: TEYVAT_256_SIFT_MAT.to_string(),
        feature_split_rows: 60,
        feature_split_cols: 88,
        prev_rect_expand_blocks: 1,
        fallback_full_search: true,
        center_match_rule: BigMapSiftMatchRule {
            operation: "Match".to_string(),
            ratio: 0.75,
            min_good_matches: 7,
            ransac_reprojection_threshold: 3.0,
        },
        rect_match_rule: BigMapSiftMatchRule {
            operation: "KnnMatchRect".to_string(),
            ratio: 0.75,
            min_good_matches: 7,
            ransac_reprojection_threshold: 3.0,
        },
    }
}
