use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    TensorRt,
    Cuda,
    Dml,
    Cpu,
    Dnnl,
    OpenVino,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferenceDeviceType {
    Cpu,
    GpuDirectMl,
    Gpu,
    OpenVino,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct OnnxProviderSelection {
    pub enable_tensor_rt_cache: bool,
    pub providers: &'static [ProviderType],
}

impl OnnxProviderSelection {
    pub const CPU: Self = Self {
        enable_tensor_rt_cache: false,
        providers: &[ProviderType::Cpu],
    };

    pub fn supports_tensor_rt(self) -> bool {
        self.providers.contains(&ProviderType::TensorRt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OnnxModel {
    pub rust_name: &'static str,
    pub legacy_registered_name: &'static str,
    pub model_relative_path: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnnxModelLoadSource {
    SourceModel,
    TensorRtAnonymousCache,
    TensorRtNamedCache,
    PreCachedModelPath,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OnnxModelLoadPlan {
    pub model_name: String,
    pub model_path: PathBuf,
    pub source_model_path: PathBuf,
    pub cache_dir: PathBuf,
    pub anonymous_tensor_rt_cache_path: PathBuf,
    pub named_tensor_rt_cache_path: PathBuf,
    pub source: OnnxModelLoadSource,
    pub exists: bool,
    pub will_generate_tensor_rt_cache: bool,
    pub message: String,
}

impl OnnxModel {
    pub fn cache_relative_path(&self, version: &str) -> PathBuf {
        Path::new("Cache")
            .join(version)
            .join("Model")
            .join(self.legacy_registered_name)
    }

    pub fn source_model_path(&self, workspace_root: impl AsRef<Path>) -> PathBuf {
        workspace_root.as_ref().join(self.model_relative_path)
    }

    pub fn cache_dir(&self, workspace_root: impl AsRef<Path>, version: &str) -> PathBuf {
        workspace_root
            .as_ref()
            .join(self.cache_relative_path(version))
    }

    pub fn load_plan(
        &self,
        workspace_root: impl AsRef<Path>,
        version: &str,
        provider_selection: OnnxProviderSelection,
    ) -> OnnxModelLoadPlan {
        let workspace_root = workspace_root.as_ref();
        let source_model_path = self.source_model_path(workspace_root);
        let cache_dir = self.cache_dir(workspace_root, version);
        let trt_dir = cache_dir.join("trt");
        let anonymous_tensor_rt_cache_path = trt_dir.join("_ctx.onnx");
        let named_tensor_rt_cache_path = trt_dir.join(format!(
            "{}_ctx.onnx",
            source_model_path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or(self.rust_name)
        ));
        let cache_prefix = self.cache_relative_path(version);
        let pre_cached = Path::new(self.model_relative_path).starts_with(&cache_prefix)
            && self.model_relative_path.ends_with("_ctx.onnx");

        let (model_path, source, will_generate_tensor_rt_cache) =
            if provider_selection.enable_tensor_rt_cache && provider_selection.supports_tensor_rt()
            {
                if pre_cached {
                    (
                        source_model_path.clone(),
                        OnnxModelLoadSource::PreCachedModelPath,
                        false,
                    )
                } else if anonymous_tensor_rt_cache_path.is_file() {
                    (
                        anonymous_tensor_rt_cache_path.clone(),
                        OnnxModelLoadSource::TensorRtAnonymousCache,
                        false,
                    )
                } else if named_tensor_rt_cache_path.is_file() {
                    (
                        named_tensor_rt_cache_path.clone(),
                        OnnxModelLoadSource::TensorRtNamedCache,
                        false,
                    )
                } else {
                    (
                        source_model_path.clone(),
                        OnnxModelLoadSource::SourceModel,
                        true,
                    )
                }
            } else {
                (
                    source_model_path.clone(),
                    OnnxModelLoadSource::SourceModel,
                    false,
                )
            };
        let exists = model_path.is_file();
        let source = if exists {
            source
        } else {
            OnnxModelLoadSource::Missing
        };
        let message = match source {
            OnnxModelLoadSource::SourceModel if will_generate_tensor_rt_cache => {
                "source model will be loaded and TensorRT cache generation may run".to_string()
            }
            OnnxModelLoadSource::SourceModel => "source model will be loaded".to_string(),
            OnnxModelLoadSource::TensorRtAnonymousCache => {
                "anonymous TensorRT cache model will be loaded".to_string()
            }
            OnnxModelLoadSource::TensorRtNamedCache => {
                "named TensorRT cache model will be loaded".to_string()
            }
            OnnxModelLoadSource::PreCachedModelPath => {
                "model path already points at a TensorRT cache model".to_string()
            }
            OnnxModelLoadSource::Missing => {
                format!("model file is missing: {}", model_path.display())
            }
        };

        OnnxModelLoadPlan {
            model_name: self.rust_name.to_string(),
            model_path,
            source_model_path,
            cache_dir,
            anonymous_tensor_rt_cache_path,
            named_tensor_rt_cache_path,
            source,
            exists,
            will_generate_tensor_rt_cache,
            message,
        }
    }
}

pub fn registered_onnx_models() -> Vec<OnnxModel> {
    vec![
        OnnxModel {
            rust_name: "YapModelTraining",
            legacy_registered_name: "YapModelTraining",
            model_relative_path: "Assets/Model/Yap/model_training.onnx",
        },
        OnnxModel {
            rust_name: "BgiFish",
            legacy_registered_name: "BgiFish",
            model_relative_path: "Assets/Model/Fish/bgi_fish.onnx",
        },
        OnnxModel {
            rust_name: "BgiTree",
            legacy_registered_name: "BgiTree",
            model_relative_path: "Assets/Model/Domain/bgi_tree.onnx",
        },
        OnnxModel {
            rust_name: "BgiWorld",
            legacy_registered_name: "BgiTree",
            model_relative_path: "Assets/Model/World/bgi_world.onnx",
        },
        OnnxModel {
            rust_name: "BgiMine",
            legacy_registered_name: "BgiMine",
            model_relative_path: "Assets/Model/Mine/bgi_mine.onnx",
        },
        OnnxModel {
            rust_name: "GridIcon",
            legacy_registered_name: "GridIcon",
            model_relative_path: "Assets/Model/Item/gridIcon.onnx",
        },
        OnnxModel {
            rust_name: "BgiAvatarSide",
            legacy_registered_name: "BgiAvatarSide",
            model_relative_path: "Assets/Model/Common/avatar_side_classify_sim.onnx",
        },
        OnnxModel {
            rust_name: "BgiQClassify",
            legacy_registered_name: "BgiQClassify",
            model_relative_path: "Assets/Model/Common/q_classify_sim.onnx",
        },
        OnnxModel {
            rust_name: "SileroVad",
            legacy_registered_name: "SileroVad",
            model_relative_path: "Assets/Model/Vad/silero_vad.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrDetV4",
            legacy_registered_name: "PpOcrDetV4",
            model_relative_path:
                "Assets/Model/PaddleOCR/Det/V4/PP-OCRv4_mobile_det_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrDetV5",
            legacy_registered_name: "PpOcrDetV5",
            model_relative_path:
                "Assets/Model/PaddleOCR/Det/V5/PP-OCRv5_mobile_det_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrDetV6",
            legacy_registered_name: "PpOcrDetV6",
            model_relative_path: "Assets/Model/PaddleOCR/Det/V6/PP-OCRv6_small_det_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV4",
            legacy_registered_name: "PpOcrRecV4",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V4/PP-OCRv4_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV4En",
            legacy_registered_name: "PpOcrRecV4En",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V4/en_PP-OCRv4_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV5",
            legacy_registered_name: "PpOcrRecV5",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V5/PP-OCRv5_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV6",
            legacy_registered_name: "PpOcrRecV6",
            model_relative_path: "Assets/Model/PaddleOCR/Rec/V6/PP-OCRv6_small_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV5Latin",
            legacy_registered_name: "PpOcrRecV5Latin",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V5/latin_PP-OCRv5_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV5Eslav",
            legacy_registered_name: "PpOcrRecV5Eslav",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V5/eslav_PP-OCRv5_mobile_rec_infer/slim.onnx",
        },
        OnnxModel {
            rust_name: "PaddleOcrRecV5Korean",
            legacy_registered_name: "PpOcrRecV5Korean",
            model_relative_path:
                "Assets/Model/PaddleOCR/Rec/V5/korean_PP-OCRv5_mobile_rec_infer/slim.onnx",
        },
    ]
}
