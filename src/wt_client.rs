use super::AnyError;
use crate::wt_error::WTError;
use magic_crypt::MagicCryptTrait;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method};
use serde::Deserialize;
use serde::Serialize;
use std::io::Write;

type ApiResult = Result<serde_json::Value, AnyError>;

const PC_CLIENT_JSON_ENCRYPTION_KEY: &'static str = "PmtIO4eCl6u11mKp";

#[derive(Debug, Serialize, Deserialize)]
pub struct WTClientConfig {
    api_endpoint: Option<String>,
    version: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    access_token: Option<String>,
    expires_in: Option<i32>,
}

impl WTClientConfig {
    pub fn new(
        api_endpoint: Option<String>,
        version: Option<String>,
        client_id: Option<String>,
        client_secret: Option<String>,
        access_token: Option<String>,
        expires_in: Option<i32>,
    ) -> WTClientConfig {
        WTClientConfig {
            api_endpoint: api_endpoint,
            version: version,
            client_id: client_id,
            client_secret: client_secret,
            access_token: access_token,
            expires_in: expires_in,
        }
    }

    pub fn load(path: &str) -> WTClientConfig {
        if let Ok(base64) = std::fs::read_to_string(path) {
            let mc = new_magic_crypt!(PC_CLIENT_JSON_ENCRYPTION_KEY, 256);
            let content = mc.decrypt_base64_to_string(&base64).unwrap();
            let config: WTClientConfig = serde_json::from_str(&content).unwrap();
            config
        } else {
            WTClientConfig {
                api_endpoint: Some(String::from("https://open.worktile.com")),
                version: Some(String::from("1")),
                client_id: None,
                client_secret: None,
                access_token: None,
                expires_in: None,
            }
        }
    }

    pub fn save(&self, path: &str) -> Result<(), AnyError> {
        let content = serde_json::to_string(self)?;
        let mc = new_magic_crypt!(PC_CLIENT_JSON_ENCRYPTION_KEY, 256);
        let base64 = mc.encrypt_str_to_base64(content);
        let mut file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(path).unwrap();
        write!(file, "{}", base64)?;
        Ok(())
    }
}

pub struct WTClient {}

#[derive(Serialize, Deserialize)]
struct AuthResponse {
    access_token: String,
    expires_in: i32,
    token_type: String,
}

pub struct Parent {
    resource: String,
    param: String,
}

impl Parent {
    pub fn new(resource: String, param: String) -> Parent {
        Parent { resource: resource, param: param }
    }
}

impl WTClient {
    fn get_client_path() -> &'static str {
        ".pc_client"
    }

    async fn request_internal(
        method: Method,
        api_endpoint: &str,
        uri: &str,
        query: Option<std::vec::Vec<(String, String)>>,
        body: Option<&serde_json::Value>,
        headers: Option<HeaderMap>,
    ) -> Result<serde_json::Value, AnyError> {
        let url = format!("{}/{}", api_endpoint, uri);
        let client = Client::new();
        let mut builder = client.request(method, &url);
        if let Some(headers) = headers {
            builder = builder.headers(headers);
        }
        if let Some(body) = body {
            builder = builder.json(body);
        }
        if let Some(queries) = query {
            builder = builder.query(&queries);
        }
        let request = builder.build()?;

        debug!("{} {} - Body = {}", request.method(), request.url(), match body {
            Some(body) => body.to_string(),
            None => String::from("None")
        });

        let res = client.execute(request).await?;
        if res.status().is_success() {
            Ok(res.json().await?)
        } else if res.status().is_client_error() {
            let err: WTError = serde_json::from_value(res.json().await?)?;
            Err(Box::new(err))
        } else {
            Err(WTError::new_boxed("000000", "Unknown Server Error"))
        }
    }

    pub async fn request(
        method: Method,
        area: Option<&str>,
        resource: &str,
        param: Option<&str>,
        parents: Option<std::vec::Vec<Parent>>,
        query: Option<std::vec::Vec<(String, String)>>,
        body: Option<&serde_json::Value>,
    ) -> ApiResult {
        // try load config and process auth if not login
        let config = WTClientConfig::load(WTClient::get_client_path());
        if config.access_token.is_none() {
            if config.client_id.is_none() || config.client_secret.is_none() || config.api_endpoint.is_none() {
                return Err(WTError::new_boxed("000000", "Please login first"));
            } else {
                WTClient::auth(
                    &config.client_id.unwrap(),
                    &config.client_secret.unwrap(),
                    &config.api_endpoint.unwrap(),
                    &config.version.unwrap(),
                )
                .await?;
            }
        }

        // load config again after auth to process the underlying request
        let config = WTClientConfig::load(WTClient::get_client_path());
        let mut headers = HeaderMap::new();
        headers.insert("authorization", format!("Bearer {}", config.access_token.unwrap()).parse().unwrap());
        headers.insert("content-type", "application/json".parse().unwrap());
        let mut uri = std::path::PathBuf::new();
        uri.push(&format!("v{}", &config.version.unwrap()));
        if let Some(area) = area {
            uri.push(area);
        }
        // build parents before resource
        if let Some(parents) = parents {
            for parent in parents.iter() {
                uri.push(parent.resource.clone());
                uri.push(parent.param.clone());
            }
        }
        uri.push(resource);
        if let Some(param) = param {
            uri.push(param);
        }
        WTClient::request_internal(method, &config.api_endpoint.unwrap(), uri.to_str().unwrap(), query, body, Some(headers)).await
    }

    pub async fn auth(client_id: &String, client_secret: &String, api_endpoint: &String, version: &String) -> Result<(), AnyError> {
        let uri = format!(
            "v{}/auth/token?grant_type=client_credentials&client_id={}&client_secret={}",
            version, client_id, client_secret
        );
        let res: AuthResponse = serde_json::from_value(WTClient::request_internal(Method::GET, &api_endpoint, &uri, None, None, None).await?)?;
        let config = WTClientConfig::new(
            Some(api_endpoint.clone()),
            Some(version.clone()),
            Some(client_id.clone()),
            Some(client_secret.clone()),
            Some(res.access_token.clone()),
            Some(res.expires_in.clone()),
        );
        config.save(WTClient::get_client_path())
    }

    pub async fn ping() -> Result<String, AnyError> {
        let res = WTClient::request(Method::GET, Some("auth"), "ping", None, None, None, None).await?;
        let res: serde_json::Value = serde_json::from_value(res)?;
        if let serde_json::Value::String(pong) = &res["data"] {
            Ok(pong.clone())
        } else {
            Err(WTError::new_boxed("000000", "Invalid ping response"))
        }
    }
}
