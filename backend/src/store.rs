use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Store {
    api_token: String,
    pub upload_dir: PathBuf,
}

impl Store {
    pub fn new(api_token: String, upload_dir: PathBuf) -> Self {
        Self {
            api_token,
            upload_dir,
        }
    }

    pub fn api_token_check(&self, auth_header: &str) -> bool {
        auth_header == format!("Bearer {}", self.api_token)
    }
}
