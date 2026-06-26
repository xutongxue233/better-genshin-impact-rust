use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, BgiError>;

#[derive(Debug, thiserror::Error)]
pub enum BgiError {
    #[error("I/O error at {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("JSON error at {path:?}: {message}")]
    Json {
        path: Option<PathBuf>,
        message: String,
    },

    #[error("asset {feature}/{name} was not found; searched {searched:?}")]
    AssetNotFound {
        feature: String,
        name: String,
        searched: Vec<PathBuf>,
    },

    #[error("workspace root does not contain BGI asset roots: {0:?}")]
    InvalidWorkspace(PathBuf),
}

impl BgiError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn json(path: Option<impl Into<PathBuf>>, message: impl Into<String>) -> Self {
        Self::Json {
            path: path.map(Into::into),
            message: message.into(),
        }
    }
}
