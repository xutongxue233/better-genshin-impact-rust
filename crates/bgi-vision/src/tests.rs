use super::*;
use std::{fs, path::Path};

#[test]
fn ocr_result_text_orders_regions_top_to_bottom_then_left_to_right() {
    let result = OcrResult {
        regions: vec![
            OcrResultRegion {
                rect: Rect::new(20, 10, 4, 4).unwrap(),
                text: "B".to_string(),
                score: 0.9,
            },
            OcrResultRegion {
                rect: Rect::new(2, 10, 4, 4).unwrap(),
                text: "A".to_string(),
                score: 0.9,
            },
            OcrResultRegion {
                rect: Rect::new(2, 30, 4, 4).unwrap(),
                text: "C".to_string(),
                score: 0.9,
            },
        ],
    };

    assert_eq!(result.text(), "A\nB\nC");
}

#[test]
fn evaluates_legacy_ocr_match_rules() {
    let mut config = OcrMatchConfig::default();
    config
        .replace_dictionary
        .insert("resin".to_string(), vec!["resln".to_string()]);
    config.all_contain_match_text = vec!["originalresin".to_string()];
    config.one_contain_match_text = vec!["160".to_string(), "200".to_string()];
    config.regex_match_text = vec![r"\d{3}".to_string()];

    assert!(config.matches_text("original resln 160").unwrap());
    assert!(!config.matches_text("original resln 20").unwrap());
}

#[test]
fn keeps_template_threshold_semantics() {
    assert!(TemplateMatchMode::CCoeffNormed
        .accepts_legacy_score(0.81, 0.8)
        .unwrap());
    assert!(TemplateMatchMode::SqDiffNormed
        .accepts_legacy_score(0.19, 0.8)
        .unwrap());
    assert!(!TemplateMatchMode::SqDiffNormed
        .accepts_legacy_score(0.21, 0.8)
        .unwrap());
}

#[test]
fn converts_bgr_to_rgb_gray_and_opencv_hsv_planes() {
    let pixels = bgr_pixels(&[[255, 0, 0], [0, 255, 0], [0, 0, 255], [10, 20, 30]]);
    let size = Size::new(4, 1);

    let rgb = convert_bgr_image(&pixels, size, ColorConversion::BgrToRgb).unwrap();
    let gray = convert_bgr_image(&pixels, size, ColorConversion::BgrToGray).unwrap();
    let hsv = convert_bgr_image(&pixels, size, ColorConversion::BgrToHsv).unwrap();

    assert_eq!(rgb.channels, 3);
    assert_eq!(
        rgb.pixels,
        vec![0, 0, 255, 0, 255, 0, 255, 0, 0, 30, 20, 10]
    );
    assert_eq!(gray.channels, 1);
    assert_eq!(gray.pixels, vec![29, 150, 76, 22]);
    assert_eq!(hsv.channel_values(0, 0), &[120, 255, 255]);
    assert_eq!(hsv.channel_values(1, 0), &[60, 255, 255]);
    assert_eq!(hsv.channel_values(2, 0), &[0, 255, 255]);
}

#[test]
fn in_range_mask_counts_roi_pixels_and_bounding_rect() {
    let image = ColorPlaneImage::new(
        Size::new(4, 2),
        3,
        vec![
            0, 0, 0, 255, 0, 0, 255, 0, 0, 0, 0, 0, //
            0, 0, 0, 255, 0, 0, 255, 0, 0, 0, 0, 0,
        ],
    )
    .unwrap();

    let mask = in_range_mask(
        &image,
        Scalar4 {
            v0: 250.0,
            v1: 0.0,
            v2: 0.0,
            v3: 0.0,
        },
        Scalar4 {
            v0: 255.0,
            v1: 5.0,
            v2: 5.0,
            v3: 0.0,
        },
        Some(Rect::new(1, 0, 2, 2).unwrap()),
    )
    .unwrap();

    assert_eq!(mask.matched_count, 4);
    assert_eq!(
        mask.bounding_rect(Some(Rect::new(1, 0, 2, 2).unwrap()))
            .unwrap(),
        Some(Rect::new(1, 0, 2, 2).unwrap())
    );
    assert_eq!(mask.pixels, vec![0, 255, 255, 0, 0, 255, 255, 0]);
}

#[test]
fn pure_rust_color_match_returns_roi_bounding_rect_and_count() {
    let image = BgrImage::new(
        Size::new(4, 3),
        bgr_pixels(&[
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [10, 20, 30],
            [10, 20, 30],
            [0, 0, 0],
            [0, 0, 0],
            [10, 20, 30],
            [10, 20, 30],
            [0, 0, 0],
        ]),
    )
    .unwrap();
    let mut object = RecognitionObject {
        recognition_type: RecognitionType::ColorMatch,
        region_of_interest: Some(Rect::new(1, 1, 2, 2).unwrap()),
        name: Some("warm-block".to_string()),
        ..RecognitionObject::default()
    };
    object.color.conversion = ColorConversion::BgrToRgb;
    object.color.lower_color = Scalar4 {
        v0: 25.0,
        v1: 15.0,
        v2: 5.0,
        v3: 0.0,
    };
    object.color.upper_color = Scalar4 {
        v0: 35.0,
        v1: 25.0,
        v2: 15.0,
        v3: 0.0,
    };
    object.color.match_count = 4;
    let backend = PureRustVisionBackend::new();

    let region = backend.find(&image.pixels, image.size, &object).unwrap();
    object.color.match_count = 5;
    let missing = backend.find(&image.pixels, image.size, &object).unwrap();

    assert_eq!(region.rect, Rect::new(1, 1, 2, 2).unwrap());
    assert_eq!(region.text, "warm-block");
    assert_eq!(region.score, Some(4.0));
    assert!(!missing.is_exist());
}

#[test]
fn image_region_crops_scales_and_runs_recognition_on_pixels() {
    let image = BgrImage::new(
        Size::new(4, 3),
        bgr_pixels(&[
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [10, 10, 10],
            [20, 20, 20],
            [0, 0, 0],
            [0, 0, 0],
            [30, 30, 30],
            [40, 40, 40],
            [0, 0, 0],
        ]),
    )
    .unwrap();
    let region = ImageRegion::capture(image)
        .derive_crop(Rect::new(1, 1, 2, 2).unwrap())
        .unwrap();
    let scaled = region.derive_to_1080p().unwrap();
    let template_path = Path::new("templates").join("block.png");
    let template = BgrImage::new(
        Size::new(2, 2),
        bgr_pixels(&[[10, 10, 10], [20, 20, 20], [30, 30, 30], [40, 40, 40]]),
    )
    .unwrap();
    let backend = PureRustVisionBackend::new().with_template(&template_path, template);
    let mut template_object = RecognitionObject::template_match(&template_path);
    template_object.template.threshold = 0.99;
    let mut color_object = RecognitionObject {
        recognition_type: RecognitionType::ColorMatch,
        ..RecognitionObject::default()
    };
    color_object.color.conversion = ColorConversion::BgrToRgb;
    color_object.color.lower_color = Scalar4 {
        v0: 35.0,
        v1: 35.0,
        v2: 35.0,
        v3: 0.0,
    };
    color_object.color.upper_color = Scalar4 {
        v0: 45.0,
        v1: 45.0,
        v2: 45.0,
        v3: 0.0,
    };

    let template_match = region.find(&backend, &template_object).unwrap();
    let color_match = region.find(&backend, &color_object).unwrap();

    assert_eq!(region.model.rect, Rect::new(1, 1, 2, 2).unwrap());
    assert_eq!(
        region.image.pixels,
        bgr_pixels(&[[10, 10, 10], [20, 20, 20], [30, 30, 30], [40, 40, 40]])
    );
    assert_eq!(scaled.image, region.image);
    assert_eq!(template_match.rect, Rect::new(0, 0, 2, 2).unwrap());
    assert_eq!(color_match.rect, Rect::new(1, 1, 1, 1).unwrap());
    assert_eq!(color_match.score, Some(1.0));
}

#[test]
fn pure_rust_template_match_finds_best_bgr24_match_inside_roi() {
    let image = BgrImage::new(
        Size::new(5, 4),
        bgr_pixels(&[
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [10, 10, 10],
            [20, 20, 20],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [30, 30, 30],
            [40, 40, 40],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
        ]),
    )
    .unwrap();
    let template = BgrImage::new(
        Size::new(2, 2),
        bgr_pixels(&[[10, 10, 10], [20, 20, 20], [30, 30, 30], [40, 40, 40]]),
    )
    .unwrap();
    let template_path = Path::new("templates").join("block.bgr");
    let backend = PureRustVisionBackend::new().with_template(&template_path, template);
    let mut object = RecognitionObject::template_match(&template_path);
    object.name = Some("block".to_string());
    object.region_of_interest = Some(Rect::new(1, 1, 3, 2).unwrap());
    object.template.threshold = 0.99;
    object.template.mode = TemplateMatchMode::CCoeffNormed;

    let region = backend.find(&image.pixels, image.size, &object).unwrap();

    assert_eq!(region.rect, Rect::new(1, 1, 2, 2).unwrap());
    assert_eq!(region.text, "block");
    assert!(region.score.unwrap() >= 0.99);
}

#[test]
fn pure_rust_template_match_applies_binary_threshold() {
    let image = BgrImage::new(
        Size::new(3, 3),
        bgr_pixels(&[
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [220, 220, 220],
            [20, 20, 20],
            [0, 0, 0],
            [20, 20, 20],
            [220, 220, 220],
        ]),
    )
    .unwrap();
    let template = BgrImage::new(
        Size::new(2, 2),
        bgr_pixels(&[[255, 255, 255], [0, 0, 0], [0, 0, 0], [255, 255, 255]]),
    )
    .unwrap();
    let template_path = Path::new("templates").join("binary-block.bgr");
    let backend = PureRustVisionBackend::new().with_template(&template_path, template);
    let mut object = RecognitionObject::template_match(&template_path);
    object.region_of_interest = Some(Rect::new(1, 1, 2, 2).unwrap());
    object.template.threshold = 0.999;
    object.template.mode = TemplateMatchMode::CCoeffNormed;
    object.template.use_binary_match = true;
    object.template.binary_threshold = 200;

    let region = backend.find(&image.pixels, image.size, &object).unwrap();

    assert_eq!(region.rect, Rect::new(1, 1, 2, 2).unwrap());
    assert!(region.score.unwrap() >= 0.999);
}

#[test]
fn pure_rust_template_match_respects_sqdiff_threshold_and_match_limit() {
    let image = BgrImage::new(
        Size::new(4, 2),
        bgr_pixels(&[
            [1, 1, 1],
            [2, 2, 2],
            [1, 1, 1],
            [2, 2, 2],
            [3, 3, 3],
            [4, 4, 4],
            [3, 3, 3],
            [4, 4, 4],
        ]),
    )
    .unwrap();
    let template = BgrImage::new(
        Size::new(2, 2),
        bgr_pixels(&[[1, 1, 1], [2, 2, 2], [3, 3, 3], [4, 4, 4]]),
    )
    .unwrap();
    let template_path = Path::new("templates").join("pair.bgr");
    let backend = PureRustVisionBackend::new().with_template(&template_path, template);
    let mut object = RecognitionObject::template_match(&template_path);
    object.template.mode = TemplateMatchMode::SqDiffNormed;
    object.template.threshold = 1.0;
    object.template.max_match_count = 1;

    let matches = backend
        .find_multi(&image.pixels, image.size, &object)
        .unwrap();

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].rect, Rect::new(0, 0, 2, 2).unwrap());
    assert_eq!(matches[0].score, Some(0.0));
}

#[test]
fn pure_rust_template_match_reports_missing_template_and_bad_buffers() {
    let backend = PureRustVisionBackend::new();
    let object = RecognitionObject::template_match("missing.bgr");

    assert!(matches!(
        backend.find(&[0, 0, 0], Size::new(1, 1), &object),
        Err(VisionError::TemplateAssetNotRegistered(_))
    ));
    assert!(matches!(
        BgrImage::new(Size::new(2, 2), vec![0; 11]).unwrap_err(),
        VisionError::InvalidImageBuffer {
            expected: 12,
            actual: 11
        }
    ));
}

#[test]
fn bgr_image_decodes_and_writes_png_as_packed_bgr() {
    let image = BgrImage::new(Size::new(2, 1), bgr_pixels(&[[1, 2, 3], [4, 5, 6]])).unwrap();
    let path = temp_path("bgr-roundtrip.png");

    image.write_png(&path).unwrap();
    let bytes = fs::read(&path).unwrap();
    let decoded_from_file = BgrImage::read(&path).unwrap();
    let decoded_from_bytes = BgrImage::decode(&bytes).unwrap();
    let _ = fs::remove_file(&path);

    assert_eq!(decoded_from_file, image);
    assert_eq!(decoded_from_bytes, image);
    assert_eq!(decoded_from_file.to_rgb_bytes(), vec![3, 2, 1, 6, 5, 4]);
}

#[test]
fn bgr_image_samples_bgr_and_rgb_pixels() {
    let image = BgrImage::new(Size::new(2, 1), bgr_pixels(&[[1, 2, 3], [4, 5, 6]])).unwrap();

    assert_eq!(
        image.bgr_pixel_at(1, 0),
        Some(BgrPixel { b: 4, g: 5, r: 6 })
    );
    assert_eq!(
        image.rgb_pixel_at(1, 0),
        Some(RgbPixel { r: 6, g: 5, b: 4 })
    );
    assert_eq!(image.bgr_pixel_at(2, 0), None);
    assert_eq!(image.rgb_pixel_at(0, 1), None);
}

#[test]
fn pure_rust_template_match_loads_template_from_rooted_asset_path() {
    let root = temp_path("template-root");
    let template_path = Path::new("GameTask")
        .join("AutoPick")
        .join("Assets")
        .join("1920x1080")
        .join("F.png");
    let absolute_template_path = root.join(&template_path);
    fs::create_dir_all(absolute_template_path.parent().unwrap()).unwrap();
    let template = BgrImage::new(
        Size::new(2, 2),
        bgr_pixels(&[[10, 20, 30], [40, 50, 60], [70, 80, 90], [100, 110, 120]]),
    )
    .unwrap();
    template.write_png(&absolute_template_path).unwrap();
    let image = BgrImage::new(
        Size::new(3, 3),
        bgr_pixels(&[
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [10, 20, 30],
            [40, 50, 60],
            [0, 0, 0],
            [70, 80, 90],
            [100, 110, 120],
        ]),
    )
    .unwrap();
    let backend = PureRustVisionBackend::new().with_template_root(&root);
    let mut object = RecognitionObject::template_match(&template_path);
    object.template.threshold = 0.99;
    object.template.mode = TemplateMatchMode::CCoeffNormed;
    object.template.use_3_channels = true;

    let region = backend.find(&image.pixels, image.size, &object).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert_eq!(region.rect, Rect::new(1, 1, 2, 2).unwrap());
    assert!(region.score.unwrap() >= 0.99);
}

#[test]
fn bv_image_preserves_legacy_feature_asset_reference() {
    let image = BvImage::new("AutoPick:F.png").unwrap();
    let roi = Rect::new(10, 20, 30, 40).unwrap();
    let object = image
        .to_recognition_object_for_screen(Some(roi), 0.91, Size::new(2560, 1440))
        .unwrap();

    assert_eq!(image.feature_name, "AutoPick");
    assert_eq!(image.asset_name, "F.png");
    assert_eq!(
        object.template.template_asset,
        Some(
            Path::new("GameTask")
                .join("AutoPick")
                .join("Assets")
                .join("2560x1440")
                .join("F.png")
        )
    );
    assert_eq!(object.name.as_deref(), Some("AutoPick:F.png"));
    assert_eq!(object.region_of_interest, Some(roi));
    assert_eq!(object.template.threshold, 0.91);

    assert!(matches!(
        BvImage::new("AutoPick").unwrap_err(),
        VisionError::InvalidBvImageAsset
    ));
}

#[test]
fn bv_locator_models_wait_retry_and_retry_action() {
    let object = RecognitionObject::ocr(Rect::new(0, 0, 200, 80).unwrap());
    let locator = BvLocator::new(object)
        .with_timeout(1_500)
        .unwrap()
        .with_retry_interval(200)
        .unwrap()
        .with_retry_action("retry-callback");

    let plan = locator.plan(BvLocatorOperation::ClickUntilDisappears, None);

    assert_eq!(plan.operation, BvLocatorOperation::ClickUntilDisappears);
    assert_eq!(plan.timeout_ms, 1_500);
    assert_eq!(plan.retry_interval_ms, 200);
    assert_eq!(plan.retry_count, 7);
    assert_eq!(plan.retry_action.as_deref(), Some("retry-callback"));

    assert!(matches!(
        BvLocator::new(RecognitionObject::default())
            .with_retry_interval(0)
            .unwrap_err(),
        VisionError::NonPositiveDuration {
            field: "retry_interval"
        }
    ));
}

#[test]
fn bv_page_models_screenshot_ocr_and_1080p_clicks() {
    let page = BvPage {
        capture_size: Size::new(2560, 1440),
        ..BvPage::default()
    };

    assert_eq!(
        page.screenshot(),
        BvPageCommand::Screenshot {
            size: Size::new(2560, 1440)
        }
    );
    assert_eq!(
        page.wait(250).unwrap(),
        BvPageCommand::Wait { milliseconds: 250 }
    );
    assert!(matches!(
        page.wait(0).unwrap_err(),
        VisionError::NonPositiveDuration {
            field: "milliseconds"
        }
    ));

    assert_eq!(
        page.click_1080p(960.0, 540.0),
        BvPageCommand::Click1080p {
            x: 960.0,
            y: 540.0,
            capture_size: Size::new(2560, 1440),
            screen_x: 1280.0,
            screen_y: 720.0
        }
    );

    let BvPageCommand::Ocr { locator } = page.ocr(Some(Rect::new(10, 20, 100, 40).unwrap())) else {
        panic!("expected OCR command");
    };
    assert_eq!(locator.operation, BvLocatorOperation::FindAll);
    assert_eq!(
        locator.recognition_object.region_of_interest,
        Some(Rect::new(10, 20, 100, 40).unwrap())
    );
}

#[test]
fn image_region_model_derives_crops_and_1080p_scale() {
    let capture = ImageRegionModel::capture(Size::new(2560, 1440));
    let crop = capture
        .derive_crop(Rect::new(2400, 1300, 400, 400).unwrap())
        .unwrap();

    assert_eq!(crop.source, ImageRegionSource::DerivedCrop);
    assert_eq!(
        crop.rect,
        Rect {
            x: 2400,
            y: 1300,
            width: 160,
            height: 140
        }
    );
    assert_eq!(
        crop.size,
        Size {
            width: 160,
            height: 140
        }
    );
    assert!(crop.owner.is_some());

    let scaled = capture.derive_to_1080p();
    assert_eq!(scaled.source, ImageRegionSource::DerivedScale);
    assert_eq!(scaled.size, Size::new(1920, 1080));
    assert!(scaled.owner.is_some());
}

#[test]
fn preserves_legacy_onnx_registry_names() {
    let models = registered_onnx_models();
    let world = models
        .iter()
        .find(|model| model.rust_name == "BgiWorld")
        .unwrap();
    assert_eq!(world.legacy_registered_name, "BgiTree");
    assert_eq!(
        world.cache_relative_path("9.9.9"),
        Path::new("Cache")
            .join("9.9.9")
            .join("Model")
            .join("BgiTree")
    );

    let grid_icon = models
        .iter()
        .find(|model| model.rust_name == "GridIcon")
        .unwrap();
    assert_eq!(grid_icon.legacy_registered_name, "GridIcon");
    assert_eq!(
        grid_icon.model_relative_path,
        "Assets/Model/Item/gridIcon.onnx"
    );
}

#[test]
fn onnx_model_load_plan_reports_missing_source_model() {
    let root = temp_path("onnx-missing");
    let model = registered_onnx_models()
        .into_iter()
        .find(|model| model.rust_name == "BgiAvatarSide")
        .unwrap();

    let plan = model.load_plan(&root, "9.9.9", OnnxProviderSelection::CPU);

    assert_eq!(plan.source, OnnxModelLoadSource::Missing);
    assert!(!plan.exists);
    assert!(plan.model_path.ends_with(
        Path::new("Assets")
            .join("Model")
            .join("Common")
            .join("avatar_side_classify_sim.onnx")
    ));
    assert!(plan.message.contains("model file is missing"));
}

#[test]
fn onnx_model_load_plan_uses_tensor_rt_cache_when_available() {
    let root = temp_path("onnx-cache");
    let model = registered_onnx_models()
        .into_iter()
        .find(|model| model.rust_name == "BgiAvatarSide")
        .unwrap();
    let named_cache = root
        .join(model.cache_relative_path("9.9.9"))
        .join("trt")
        .join("avatar_side_classify_sim_ctx.onnx");
    fs::create_dir_all(named_cache.parent().unwrap()).unwrap();
    fs::write(&named_cache, b"fake").unwrap();

    let plan = model.load_plan(
        &root,
        "9.9.9",
        OnnxProviderSelection {
            enable_tensor_rt_cache: true,
            providers: &[ProviderType::TensorRt, ProviderType::Cpu],
        },
    );
    let _ = fs::remove_dir_all(&root);

    assert_eq!(plan.source, OnnxModelLoadSource::TensorRtNamedCache);
    assert_eq!(plan.model_path, named_cache);
    assert!(plan.exists);
    assert!(!plan.will_generate_tensor_rt_cache);
}

#[test]
fn onnx_model_load_plan_prefers_anonymous_tensor_rt_cache() {
    let root = temp_path("onnx-cache-anonymous");
    let model = registered_onnx_models()
        .into_iter()
        .find(|model| model.rust_name == "BgiAvatarSide")
        .unwrap();
    let cache_dir = root.join(model.cache_relative_path("9.9.9")).join("trt");
    let anonymous_cache = cache_dir.join("_ctx.onnx");
    let named_cache = cache_dir.join("avatar_side_classify_sim_ctx.onnx");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::write(&anonymous_cache, b"fake").unwrap();
    fs::write(&named_cache, b"fake").unwrap();

    let plan = model.load_plan(
        &root,
        "9.9.9",
        OnnxProviderSelection {
            enable_tensor_rt_cache: true,
            providers: &[ProviderType::TensorRt, ProviderType::Cpu],
        },
    );
    let _ = fs::remove_dir_all(&root);

    assert_eq!(plan.source, OnnxModelLoadSource::TensorRtAnonymousCache);
    assert_eq!(plan.model_path, anonymous_cache);
}

#[test]
fn onnx_model_load_plan_falls_back_to_source_without_tensor_rt_provider() {
    let root = temp_path("onnx-source");
    let model = registered_onnx_models()
        .into_iter()
        .find(|model| model.rust_name == "BgiAvatarSide")
        .unwrap();
    let source = root.join(model.model_relative_path);
    let cache = root
        .join(model.cache_relative_path("9.9.9"))
        .join("trt")
        .join("_ctx.onnx");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, b"fake").unwrap();
    fs::create_dir_all(cache.parent().unwrap()).unwrap();
    fs::write(&cache, b"fake").unwrap();

    let plan = model.load_plan(
        &root,
        "9.9.9",
        OnnxProviderSelection {
            enable_tensor_rt_cache: true,
            providers: &[ProviderType::Dml, ProviderType::Cpu],
        },
    );
    let _ = fs::remove_dir_all(&root);

    assert_eq!(plan.source, OnnxModelLoadSource::SourceModel);
    assert_eq!(plan.model_path, source);
    assert!(plan.exists);
    assert!(!plan.will_generate_tensor_rt_cache);
}

#[test]
fn grid_cells_cluster_rows_columns_and_fill_missing_phantoms() {
    let rects = vec![
        Rect::new(0, 0, 80, 100).unwrap(),
        Rect::new(0, 120, 80, 100).unwrap(),
        Rect::new(100, 120, 80, 100).unwrap(),
    ];

    let mut cells = cluster_grid_cells(&rects, 20);
    cells.sort_by_key(|cell| (cell.row, cell.col));

    assert_eq!(cells.len(), 4);
    assert_eq!((cells[0].row, cells[0].col), (0, 0));
    assert!(!cells[0].is_phantom);
    assert_eq!((cells[1].row, cells[1].col), (0, 1));
    assert!(cells[1].is_phantom);
    assert_eq!(cells[1].rect, Rect::new(100, 0, 80, 100).unwrap());
    assert_eq!((cells[2].row, cells[2].col), (1, 0));
    assert_eq!((cells[3].row, cells[3].col), (1, 1));

    let tied = assign_grid_cell_rows_and_columns(
        &[
            Rect::new(0, 0, 80, 100).unwrap(),
            Rect::new(250, 0, 80, 100).unwrap(),
        ],
        20,
    );
    assert_eq!(tied[1].col, 2);
}

#[test]
fn grid_post_process_filters_phantom_cells_by_legacy_bottom_color() {
    let rects = vec![
        Rect::new(0, 0, 80, 100).unwrap(),
        Rect::new(0, 120, 80, 100).unwrap(),
        Rect::new(100, 120, 80, 100).unwrap(),
    ];
    let target = BgrPixel {
        b: 0xdc,
        g: 0xe5,
        r: 0xe9,
    };
    let image = solid_bgr_image(Size::new(220, 240), [target.b, target.g, target.r]);

    let cells = post_process_grid_cells(&image, &rects, 20, target, 30).unwrap();
    assert_eq!(cells.len(), 4);
    assert!(cells.iter().any(|cell| cell.is_phantom));

    let black = solid_bgr_image(Size::new(220, 240), [0, 0, 0]);
    let cells = post_process_grid_cells(&black, &rects, 20, target, 30).unwrap();
    assert_eq!(cells.len(), 3);
    assert!(!cells.iter().any(|cell| cell.is_phantom));
}

#[test]
fn grid_icon_and_count_text_crops_follow_legacy_ratios() {
    let image = patterned_bgr_image(Size::new(250, 306));
    let crops = crop_grid_icon(&image, GridIconCropSpec::legacy_inventory()).unwrap();

    assert_eq!(crops.normalized.size, Size::new(125, 153));
    assert_eq!(crops.icon.size, Size::new(125, 125));
    assert_eq!(crops.bottom.size, Size::new(125, 27));

    let text = crop_grid_item_count_text(
        &crops.normalized,
        GridItemTextCropSpec::legacy_inventory_count(),
    )
    .unwrap();
    assert_eq!(text.size, Size::new(230, 44));
}

#[test]
fn grid_scroll_shift_rejects_identical_and_blank_frames_but_detects_translation() {
    assert!(!classify_grid_scroll_response(0.5, 0.5, 0.95));
    assert!(classify_grid_scroll_response(0.51, 0.5, 0.95));
    assert!(!classify_grid_scroll_response(0.95, 0.5, 0.95));

    let previous = patterned_bgr_image(Size::new(6, 6));
    let identical = detect_grid_scroll_shift(&previous, &previous, 0.5, 0.95, 2).unwrap();
    assert!(!identical.is_scrolling);
    assert!(identical.response >= 0.95);

    let blank = solid_bgr_image(Size::new(6, 6), [0, 0, 0]);
    let blank_decision = detect_grid_scroll_shift(&blank, &blank, 0.5, 0.95, 2).unwrap();
    assert!(!blank_decision.is_scrolling);
    assert_eq!(blank_decision.response, 0.0);

    let shifted = shift_bgr_image(&previous, 0, -1, [0, 0, 0]);
    let decision = detect_grid_scroll_shift(&previous, &shifted, 0.5, 0.95, 2).unwrap();
    assert!(decision.is_scrolling);
    assert_eq!((decision.shift_x, decision.shift_y), (0, -1));
    assert!(decision.response > 0.5 && decision.response < 0.95);
}

fn bgr_pixels(values: &[[u8; 3]]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|pixel| pixel.iter().copied())
        .collect()
}

fn solid_bgr_image(size: Size, pixel: [u8; 3]) -> BgrImage {
    let mut pixels = Vec::with_capacity(size.width as usize * size.height as usize * 3);
    for _ in 0..size.width as usize * size.height as usize {
        pixels.extend_from_slice(&pixel);
    }
    BgrImage::new(size, pixels).unwrap()
}

fn patterned_bgr_image(size: Size) -> BgrImage {
    let mut pixels = Vec::with_capacity(size.width as usize * size.height as usize * 3);
    for y in 0..size.height {
        for x in 0..size.width {
            let value = ((x * 37 + y * 53 + (x * y * 11)) % 251) as u8;
            pixels.extend_from_slice(&[value, value, value]);
        }
    }
    BgrImage::new(size, pixels).unwrap()
}

fn shift_bgr_image(image: &BgrImage, shift_x: i32, shift_y: i32, fill: [u8; 3]) -> BgrImage {
    let mut pixels = Vec::with_capacity(image.pixels.len());
    for y in 0..image.size.height as i32 {
        for x in 0..image.size.width as i32 {
            let source_x = x - shift_x;
            let source_y = y - shift_y;
            if source_x >= 0
                && source_y >= 0
                && source_x < image.size.width as i32
                && source_y < image.size.height as i32
            {
                let index = ((source_y as u32 * image.size.width + source_x as u32) as usize) * 3;
                pixels.extend_from_slice(&image.pixels[index..index + 3]);
            } else {
                pixels.extend_from_slice(&fill);
            }
        }
    }
    BgrImage::new(image.size, pixels).unwrap()
}

fn temp_path(name: &str) -> PathBuf {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("bgi-vision-{id}-{name}"))
}
