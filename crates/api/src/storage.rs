use std::{env, path::PathBuf, time::Duration};

use hmac::{Hmac, Mac};
use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageBackendConfig {
    Local { root: PathBuf },
    S3 { bucket: String, prefix: String },
}

#[derive(Debug, Clone)]
pub struct ObjectStorage {
    config: StorageBackendConfig,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage key may not be empty")]
    EmptyKey,
    #[error("storage key may not contain parent traversal")]
    InvalidKey,
    #[error("OPENGITHUB_BLOB_STORAGE must be local or s3")]
    InvalidBackend,
    #[error("OPENGITHUB_S3_BUCKET or S3_BUCKET is required when OPENGITHUB_BLOB_STORAGE=s3")]
    MissingS3Bucket,
    #[error("AWS credentials are required for S3 storage")]
    MissingAwsCredentials,
    #[error("local storage error")]
    Io(#[from] std::io::Error),
    #[error("s3 storage error: {0}")]
    S3(String),
}

#[derive(Debug, Clone)]
struct AwsCredentials {
    access_key: String,
    secret_key: String,
    session_token: Option<String>,
}

impl ObjectStorage {
    pub fn from_env_with_local(default_root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let backend = env::var("OPENGITHUB_BLOB_STORAGE")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                if crate::config::is_deployed_env() {
                    "s3".to_owned()
                } else {
                    "local".to_owned()
                }
            });
        match backend.trim().to_ascii_lowercase().as_str() {
            "local" | "filesystem" | "fs" => Ok(Self {
                config: StorageBackendConfig::Local {
                    root: default_root.into(),
                },
            }),
            "s3" => {
                let bucket = non_empty_env("OPENGITHUB_S3_BUCKET")
                    .or_else(|| non_empty_env("S3_BUCKET"))
                    .ok_or(StorageError::MissingS3Bucket)?;
                let prefix =
                    non_empty_env("OPENGITHUB_S3_PREFIX").unwrap_or_else(|| "api".to_owned());
                Ok(Self {
                    config: StorageBackendConfig::S3 {
                        bucket,
                        prefix: sanitize_prefix(&prefix),
                    },
                })
            }
            _ => Err(StorageError::InvalidBackend),
        }
    }

    pub fn config(&self) -> &StorageBackendConfig {
        &self.config
    }

    pub fn storage_kind(&self) -> &'static str {
        match self.config {
            StorageBackendConfig::Local { .. } => "local",
            StorageBackendConfig::S3 { .. } => "s3",
        }
    }

    pub async fn put(&self, key: &str, bytes: impl Into<Vec<u8>>) -> Result<(), StorageError> {
        let key = validate_key(key)?;
        let bytes = bytes.into();
        match &self.config {
            StorageBackendConfig::Local { root } => {
                let path = root.join(key);
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::write(path, bytes).await?;
                Ok(())
            }
            StorageBackendConfig::S3 { bucket, prefix } => {
                s3_request("PUT", bucket, &s3_key(prefix, key), Some(bytes), None).await?;
                Ok(())
            }
        }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let key = validate_key(key)?;
        match &self.config {
            StorageBackendConfig::Local { root } => Ok(tokio::fs::read(root.join(key)).await?),
            StorageBackendConfig::S3 { bucket, prefix } => {
                s3_request("GET", bucket, &s3_key(prefix, key), None, None).await
            }
        }
    }

    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let key = validate_key(key)?;
        match &self.config {
            StorageBackendConfig::Local { root } => {
                match tokio::fs::remove_file(root.join(key)).await {
                    Ok(()) => Ok(()),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    Err(error) => Err(StorageError::Io(error)),
                }
            }
            StorageBackendConfig::S3 { bucket, prefix } => {
                s3_request("DELETE", bucket, &s3_key(prefix, key), None, None).await?;
                Ok(())
            }
        }
    }

    pub async fn signed_get_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<Option<String>, StorageError> {
        let key = validate_key(key)?;
        match &self.config {
            StorageBackendConfig::Local { .. } => Ok(None),
            StorageBackendConfig::S3 { bucket, prefix } => Ok(Some(
                presigned_get_url(bucket, &s3_key(prefix, key), expires_in).await?,
            )),
        }
    }
}

pub fn storage_config_from_env(
    default_root: impl Into<PathBuf>,
) -> Result<StorageBackendConfig, StorageError> {
    ObjectStorage::from_env_with_local(default_root).map(|storage| storage.config)
}

async fn s3_request(
    method: &str,
    bucket: &str,
    key: &str,
    body: Option<Vec<u8>>,
    extra_query: Option<&str>,
) -> Result<Vec<u8>, StorageError> {
    let credentials = aws_credentials().await?;
    let region = non_empty_env("AWS_REGION").unwrap_or_else(|| "us-east-1".to_owned());
    let host = format!("{bucket}.s3.{region}.amazonaws.com");
    let path = format!("/{}", percent_encode_path(key));
    let endpoint = if let Some(query) = extra_query {
        format!("https://{host}{path}?{query}")
    } else {
        format!("https://{host}{path}")
    };
    let payload = body.unwrap_or_default();
    let payload_hash = hex_sha256(&payload);
    let now = chrono::Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date = now.format("%Y%m%d").to_string();
    let mut signed_headers = "host;x-amz-content-sha256;x-amz-date".to_owned();
    let mut canonical_headers =
        format!("host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n");
    if let Some(token) = &credentials.session_token {
        signed_headers.push_str(";x-amz-security-token");
        canonical_headers.push_str(&format!("x-amz-security-token:{token}\n"));
    }
    let canonical_request = format!(
        "{method}\n{path}\n{}\n{canonical_headers}\n{signed_headers}\n{payload_hash}",
        extra_query.unwrap_or("")
    );
    let credential_scope = format!("{date}/{region}/s3/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        hex_sha256(canonical_request.as_bytes())
    );
    let signature = hex_hmac(
        &signing_key(&credentials.secret_key, &date, &region),
        string_to_sign.as_bytes(),
    );
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
        credentials.access_key
    );
    let request_method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|error| StorageError::S3(error.to_string()))?;
    let client = reqwest::Client::new();
    let mut request = client
        .request(request_method.clone(), endpoint)
        .header("x-amz-date", amz_date)
        .header("x-amz-content-sha256", payload_hash)
        .header("authorization", authorization);
    if let Some(token) = credentials.session_token {
        request = request.header("x-amz-security-token", token);
    }
    if !payload.is_empty() {
        request = request.body(payload);
    }
    let response = request
        .send()
        .await
        .map_err(|error| StorageError::S3(error.to_string()))?;
    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| StorageError::S3(error.to_string()))?;
    if !(status.is_success()
        || request_method == reqwest::Method::DELETE && status == StatusCode::NOT_FOUND)
    {
        return Err(StorageError::S3(format!(
            "S3 {method} {key} failed with {status}"
        )));
    }
    Ok(bytes.to_vec())
}

async fn presigned_get_url(
    bucket: &str,
    key: &str,
    expires_in: Duration,
) -> Result<String, StorageError> {
    let credentials = aws_credentials().await?;
    let region = non_empty_env("AWS_REGION").unwrap_or_else(|| "us-east-1".to_owned());
    let host = format!("{bucket}.s3.{region}.amazonaws.com");
    let path = format!("/{}", percent_encode_path(key));
    let now = chrono::Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date = now.format("%Y%m%d").to_string();
    let credential_scope = format!("{date}/{region}/s3/aws4_request");
    let credential = format!("{}/{}", credentials.access_key, credential_scope);
    let mut pairs = vec![
        ("X-Amz-Algorithm".to_owned(), "AWS4-HMAC-SHA256".to_owned()),
        ("X-Amz-Credential".to_owned(), credential),
        ("X-Amz-Date".to_owned(), amz_date.clone()),
        (
            "X-Amz-Expires".to_owned(),
            expires_in.as_secs().min(604800).to_string(),
        ),
        ("X-Amz-SignedHeaders".to_owned(), "host".to_owned()),
    ];
    if let Some(token) = &credentials.session_token {
        pairs.push(("X-Amz-Security-Token".to_owned(), token.clone()));
    }
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let canonical_query = pairs
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let canonical_request =
        format!("GET\n{path}\n{canonical_query}\nhost:{host}\n\nhost\nUNSIGNED-PAYLOAD");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        hex_sha256(canonical_request.as_bytes())
    );
    let signature = hex_hmac(
        &signing_key(&credentials.secret_key, &date, &region),
        string_to_sign.as_bytes(),
    );
    Ok(format!(
        "https://{host}{path}?{canonical_query}&X-Amz-Signature={signature}"
    ))
}

async fn aws_credentials() -> Result<AwsCredentials, StorageError> {
    if let (Some(access_key), Some(secret_key)) = (
        non_empty_env("AWS_ACCESS_KEY_ID"),
        non_empty_env("AWS_SECRET_ACCESS_KEY"),
    ) {
        return Ok(AwsCredentials {
            access_key,
            secret_key,
            session_token: non_empty_env("AWS_SESSION_TOKEN"),
        });
    }
    if let Some(relative_uri) = non_empty_env("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI") {
        let url = format!("http://169.254.170.2{relative_uri}");
        let value: serde_json::Value = reqwest::get(url)
            .await
            .map_err(|error| StorageError::S3(error.to_string()))?
            .json()
            .await
            .map_err(|error| StorageError::S3(error.to_string()))?;
        let access_key = value
            .get("AccessKeyId")
            .and_then(|v| v.as_str())
            .ok_or(StorageError::MissingAwsCredentials)?
            .to_owned();
        let secret_key = value
            .get("SecretAccessKey")
            .and_then(|v| v.as_str())
            .ok_or(StorageError::MissingAwsCredentials)?
            .to_owned();
        let session_token = value
            .get("Token")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        return Ok(AwsCredentials {
            access_key,
            secret_key,
            session_token,
        });
    }
    Err(StorageError::MissingAwsCredentials)
}

fn signing_key(secret_key: &str, date: &str, region: &str) -> Vec<u8> {
    let k_date = hmac_bytes(format!("AWS4{secret_key}").as_bytes(), date.as_bytes());
    let k_region = hmac_bytes(&k_date, region.as_bytes());
    let k_service = hmac_bytes(&k_region, b"s3");
    hmac_bytes(&k_service, b"aws4_request")
}

fn hmac_bytes(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn hex_hmac(key: &[u8], data: &[u8]) -> String {
    hmac_bytes(key, data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn percent_encode_path(value: &str) -> String {
    value
        .split('/')
        .map(percent_encode)
        .collect::<Vec<_>>()
        .join("/")
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn non_empty_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

fn sanitize_prefix(prefix: &str) -> String {
    prefix.trim_matches('/').to_owned()
}

fn s3_key(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_owned()
    } else {
        format!("{prefix}/{key}")
    }
}

fn validate_key(key: &str) -> Result<&str, StorageError> {
    let key = key.trim_matches('/');
    if key.is_empty() {
        return Err(StorageError::EmptyKey);
    }
    if key
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        || key.contains('\\')
    {
        return Err(StorageError::InvalidKey);
    }
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use uuid::Uuid;

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn with_storage_env(test: impl FnOnce()) {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let names = [
            "OPENGITHUB_BLOB_STORAGE",
            "OPENGITHUB_S3_BUCKET",
            "S3_BUCKET",
            "OPENGITHUB_S3_PREFIX",
            "AWS_ACCESS_KEY_ID",
            "AWS_SECRET_ACCESS_KEY",
            "AWS_SESSION_TOKEN",
            "APP_ENV",
            "ENVIRONMENT",
            "NODE_ENV",
            "RAILS_ENV",
        ];
        let original: Vec<_> = names
            .iter()
            .map(|name| (*name, env::var(name).ok()))
            .collect();
        for name in names {
            env::remove_var(name);
        }
        test();
        for (name, value) in original {
            match value {
                Some(value) => env::set_var(name, value),
                None => env::remove_var(name),
            }
        }
    }

    #[test]
    fn local_storage_is_default_for_dev() {
        with_storage_env(|| {
            let config = storage_config_from_env("/tmp/opengithub-test").unwrap();
            assert_eq!(
                config,
                StorageBackendConfig::Local {
                    root: PathBuf::from("/tmp/opengithub-test")
                }
            );
        });
    }

    #[test]
    fn production_defaults_to_s3_and_requires_bucket() {
        with_storage_env(|| {
            env::set_var("APP_ENV", "production");
            let error = storage_config_from_env("/tmp/opengithub-test")
                .unwrap_err()
                .to_string();
            assert!(error.contains("OPENGITHUB_S3_BUCKET"));
        });
    }

    #[test]
    fn s3_config_accepts_bucket_and_prefix() {
        with_storage_env(|| {
            env::set_var("OPENGITHUB_BLOB_STORAGE", "s3");
            env::set_var("OPENGITHUB_S3_BUCKET", "opengithub-storage");
            env::set_var("OPENGITHUB_S3_PREFIX", "/api-staging/");
            let config = storage_config_from_env("/tmp/opengithub-test").unwrap();
            assert_eq!(
                config,
                StorageBackendConfig::S3 {
                    bucket: "opengithub-storage".to_owned(),
                    prefix: "api-staging".to_owned()
                }
            );
        });
    }

    #[test]
    fn local_put_get_delete_roundtrip() {
        with_storage_env(|| {
            let root = env::temp_dir().join(format!("opengithub-storage-test-{}", Uuid::new_v4()));
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let storage = ObjectStorage::from_env_with_local(&root).unwrap();
                storage
                    .put("packages/blob.txt", b"hello".to_vec())
                    .await
                    .unwrap();
                assert_eq!(storage.get("packages/blob.txt").await.unwrap(), b"hello");
                storage.delete("packages/blob.txt").await.unwrap();
                assert!(storage.get("packages/blob.txt").await.is_err());
                let _ = tokio::fs::remove_dir_all(root).await;
            });
        });
    }

    #[test]
    fn s3_presigned_url_smoke_does_not_call_aws() {
        with_storage_env(|| {
            env::set_var("OPENGITHUB_BLOB_STORAGE", "s3");
            env::set_var("OPENGITHUB_S3_BUCKET", "opengithub-storage");
            env::set_var("OPENGITHUB_S3_PREFIX", "api");
            env::set_var("AWS_REGION", "us-west-2");
            env::set_var("AWS_ACCESS_KEY_ID", "AKIAEXAMPLE");
            env::set_var("AWS_SECRET_ACCESS_KEY", "secret");
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let storage = ObjectStorage::from_env_with_local("/tmp/unused").unwrap();
                let url = storage
                    .signed_get_url("packages/blob.txt", Duration::from_secs(60))
                    .await
                    .unwrap()
                    .unwrap();
                assert!(url.starts_with(
                    "https://opengithub-storage.s3.us-west-2.amazonaws.com/api/packages/blob.txt?"
                ));
                assert!(url.contains("X-Amz-Signature="));
            });
        });
    }

    #[test]
    fn traversal_keys_are_rejected() {
        assert!(validate_key("../secret").is_err());
        assert!(validate_key("packages/../secret").is_err());
        assert!(validate_key("packages\\secret").is_err());
    }
}
