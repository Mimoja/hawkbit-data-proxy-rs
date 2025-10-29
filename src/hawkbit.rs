use dotenv::dotenv;
use reqwest::Client;
use reqwest::{StatusCode, header};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct HawkbitConfig {
    host: String,
    username: String,
    password: String,
    channel: Option<String>,
}

impl HawkbitConfig {
    pub fn from_env() -> Self {
        dotenv().ok(); // Load from .env file into environment variables

        let host = env::var("HAWKBIT_HOST").unwrap_or_else(|_| "http://localhost".to_string());

        let username = env::var("HAWKBIT_USERNAME").unwrap();
        let password = env::var("HAWKBIT_PASSWORD").unwrap();
        let channel = env::var("HAWKBIT_CHANNEL").ok();

        HawkbitConfig {
            host,
            username,
            password,
            channel,
        }
    }
}

#[derive(Debug)]
pub struct HawkbitError {
    msg: String,
}

impl HawkbitError {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        Self { msg: msg.into() }
    }
}

impl fmt::Display for HawkbitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for HawkbitError {}

impl From<reqwest::Error> for HawkbitError {
    fn from(err: reqwest::Error) -> Self {
        HawkbitError::new(err.to_string())
    }
}

pub type HawkbitResult<T> = std::result::Result<T, HawkbitError>;

#[derive(Deserialize, Debug)]
pub struct MgmtTarget {
    #[serde(rename = "_links")]
    pub links: Value,

    #[serde(rename = "controllerId")]
    pub controller_id: String,
    #[serde(rename = "group")]
    pub group: Option<String>,
    #[serde(rename = "updateStatus")]
    pub update_status: Option<String>,
    #[serde(rename = "lastControllerRequestAt")]
    pub last_controller_request_at: Option<i64>,
    #[serde(rename = "installedAt")]
    pub installed_at: Option<i64>,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(rename = "address")]
    pub address: Option<String>,
    #[serde(rename = "pollStatus")]
    pub poll_status: Option<Value>,
    #[serde(rename = "securityToken")]
    pub security_token: Option<String>,
    #[serde(rename = "requestAttributes")]
    pub request_attributes: Option<bool>,
    #[serde(rename = "targetType")]
    pub target_type: Option<i64>,
    #[serde(rename = "targetTypeName")]
    pub target_type_name: Option<String>,
    #[serde(rename = "autoConfirmActive")]
    pub auto_confirm_active: Option<bool>,
}

#[derive(Debug)]
pub struct HawkbitMgmtClient {
    config: HawkbitConfig,
    client: Client,
    default_headers: header::HeaderMap,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    #[serde(rename = "_links")]
    pub links: Value,

    #[serde(rename = "createdAt")]
    pub created_at: u64,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "detailStatus")]
    pub detail_status: String,

    #[serde(rename = "forceType")]
    pub force_type: String,

    pub id: i64,

    #[serde(rename = "lastModifiedAt")]
    pub last_modified_at: u64,

    #[serde(rename = "lastModifiedBy")]
    pub last_modified_by: String,

    pub status: String,

    #[serde(rename = "type")]
    pub action_type: String,

    pub rollout: Option<u64>,

    #[serde(rename = "rolloutName")]
    pub rollout_name: Option<String>,

    pub weight: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    #[serde(rename = "href")]
    pub href: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedLink {
    #[serde(rename = "href")]
    pub href: Option<String>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "self")]
    pub self_link: Option<Link>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HawkbitActionLinks {
    #[serde(rename = "distributionset")]
    pub distribution_set: Option<NamedLink>,

    #[serde(rename = "rollout")]
    pub rollout: Option<NamedLink>,

    #[serde(rename = "self")]
    pub self_link: Option<Link>,

    #[serde(rename = "status")]
    pub status: Option<Link>,

    #[serde(rename = "target")]
    pub target: Option<NamedLink>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionDetail {
    #[serde(rename = "_links")]
    pub links: HawkbitActionLinks,

    #[serde(rename = "createdAt")]
    pub created_at: u64,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "detailStatus")]
    pub detail_status: String,

    #[serde(rename = "forceType")]
    pub force_type: String,

    #[serde(rename = "id")]
    pub id: u64,

    #[serde(rename = "lastModifiedAt")]
    pub last_modified_at: u64,

    #[serde(rename = "lastModifiedBy")]
    pub last_modified_by: String,

    #[serde(rename = "rollout")]
    pub rollout: Option<u64>,

    #[serde(rename = "rolloutName")]
    pub rollout_name: Option<String>,

    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "type")]
    pub action_type: String, // `type` is reserved in Rust
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionStatusEvent {
    #[serde(rename = "id")]
    pub id: u64,

    #[serde(rename = "messages")]
    pub messages: Vec<String>,

    #[serde(rename = "reportedAt")]
    pub reported_at: u64,

    #[serde(rename = "type")]
    pub event_type: String, // renamed because `type` is a reserved keyword
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoftwareModuleLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoftwareModule {
    #[serde(rename = "_links")]
    pub links: SoftwareModuleLinks,

    #[serde(rename = "createdAt")]
    pub created_at: u64,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    pub deleted: bool,
    pub encrypted: bool,
    pub id: u64,

    #[serde(rename = "lastModifiedAt")]
    pub last_modified_at: u64,

    #[serde(rename = "lastModifiedBy")]
    pub last_modified_by: String,

    pub name: String,

    #[serde(rename = "type")]
    pub module_type: String,

    #[serde(rename = "typeName")]
    pub type_name: String,

    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistributionSetLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistributionSet {
    #[serde(rename = "_links")]
    pub links: DistributionSetLinks,

    pub complete: bool,

    #[serde(rename = "createdAt")]
    pub created_at: u64,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    pub deleted: bool,
    pub description: String,
    pub id: u64,

    #[serde(rename = "lastModifiedAt")]
    pub last_modified_at: u64,

    #[serde(rename = "lastModifiedBy")]
    pub last_modified_by: String,

    pub modules: Vec<SoftwareModule>,

    pub name: String,

    #[serde(rename = "requiredMigrationStep")]
    pub required_migration_step: bool,

    #[serde(rename = "type")]
    pub ds_type: String,

    #[serde(rename = "typeName")]
    pub type_name: String,

    pub valid: bool,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationResponse<T> {
    pub content: T,
    pub size: usize,
    pub total: usize,
}

impl HawkbitMgmtClient {
    pub fn from_config(config: &HawkbitConfig) -> Self {
        let mut headers = header::HeaderMap::new();

        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers.clone())
            .build()
            .expect("Failed to build HTTP client");

        Self {
            config: config.clone(),
            client,
            default_headers: headers,
        }
    }

    fn build_url(&self, endpoint: &str) -> String {
        self.config.host.trim_end_matches('/').to_string()
            + "/rest/v1/"
            + endpoint.trim_start_matches('/')
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query_params: Option<HashMap<String, String>>,
    ) -> HawkbitResult<T> {
        let url = self.build_url(endpoint);
        let mut req = self
            .client
            .get(&url)
            .headers(self.default_headers.clone())
            .basic_auth(&self.config.username, Some(&self.config.password));
        if let Some(params) = query_params {
            for (k, v) in params {
                req = req.query(&[(k.to_string(), v.to_string())]);
            }
        }
        let res = req.send().await?;

        let status = res.status();
        if status != StatusCode::OK {
            let body = res.text().await.unwrap();
            return Err(HawkbitError::new(format!(
                "HTTP error {}: {}",
                status.as_u16(),
                body
            )));
        }

        Ok(res.json::<T>().await?)
    }

    pub async fn delete(
        &self,
        endpoint: &str,
        query_params: Option<HashMap<String, String>>,
    ) -> Result<String, HawkbitError> {
        let url = self.build_url(endpoint);
        let mut req = self
            .client
            .delete(&url)
            .headers(self.default_headers.clone())
            .basic_auth(&self.config.username, Some(&self.config.password));
        if let Some(params) = query_params {
            for (k, v) in params {
                req = req.query(&[(k.to_string(), v.to_string())]);
            }
        }
        let res = req.send().await?;

        let status = res.status();
        if status == StatusCode::NO_CONTENT {
            return Ok("No Content".to_string());
        }
        if status != StatusCode::OK {
            let body = res.text().await.unwrap();
            return Err(HawkbitError::new(format!(
                "HTTP error {}: {}",
                status.as_u16(),
                body
            )));
        }

        Ok("OK".to_string())
    }
    pub async fn post<T: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        json_data: &T,
    ) -> HawkbitResult<Value> {
        let url = self.build_url(endpoint);
        let res = self
            .client
            .post(&url)
            .headers(self.default_headers.clone())
            .basic_auth(&self.config.username, Some(&self.config.password))
            .json(json_data)
            .send()
            .await?;

        let status = res.status();
        if status != StatusCode::OK && status != StatusCode::CREATED {
            let body = res.text().await.unwrap();
            return Err(HawkbitError::new(format!(
                "HTTP error {}: {}",
                status.as_u16(),
                body
            )));
        }

        Ok(res.json().await?)
    }

    pub async fn put<T: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        json_data: &T,
    ) -> HawkbitResult<Option<Value>> {
        let url = self.build_url(endpoint);
        let res = self
            .client
            .put(&url)
            .headers(self.default_headers.clone())
            .basic_auth(&self.config.username, Some(&self.config.password))
            .json(json_data)
            .send()
            .await?;

        let status = res.status();
        if status != StatusCode::OK && status != StatusCode::NO_CONTENT {
            let body = res.text().await.unwrap();
            return Err(HawkbitError::new(format!(
                "HTTP error {}: {}",
                status.as_u16(),
                body
            )));
        }

        if status == StatusCode::NO_CONTENT {
            Ok(None)
        } else {
            Ok(Some(res.json().await?))
        }
    }

    pub async fn get_targets(&self, filter_query: Option<&str>) -> HawkbitResult<Vec<MgmtTarget>> {
        let mut offset = 0;
        let mut total = usize::MAX; // Will be overwritten on first request
        let mut targets = Vec::new();
        while offset < total {
            // Construct query parameters for pagination and filtering
            let mut query_params = HashMap::new();
            if let Some(filter) = filter_query {
                query_params.insert("q".to_string(), filter.to_string());
            }
            query_params.insert("offset".to_string(), offset.to_string());
            query_params.insert("limit".to_string(), 50.to_string());

            let new_page = self
                .get::<PaginationResponse<Vec<MgmtTarget>>>("/targets", Some(query_params))
                .await?;

            total = new_page.total;
            targets.extend(new_page.content);
            if new_page.size == 0 {
                break;
            }

            offset += new_page.size;
        }
        Ok(targets)
    }

    pub async fn get_target(&self, target_id: &str) -> HawkbitResult<MgmtTarget> {
        let endpoint = &format!("targets/{}", target_id);
        self.get::<MgmtTarget>(endpoint, None).await
    }

    pub async fn delete_target(&self, target_id: &str) -> HawkbitResult<String> {
        let endpoint = &format!("targets/{}", target_id);
        self.delete(endpoint, None).await
    }

    pub async fn get_target_actions(
        &self,
        target_id: &str,
        limit: Option<usize>,
        filter_query: Option<&str>,
    ) -> HawkbitResult<Vec<Action>> {
        let mut query_params = HashMap::new();

        query_params.insert("sort".to_string(), "id:DESC".to_string());
        query_params.insert("limit".to_string(), limit.unwrap_or(10).to_string());
        if let Some(filter_query) = filter_query {
            query_params.insert("q".to_string(), filter_query.to_string());
        }
        let endpoint = &format!("targets/{}/actions", target_id);

        let resp = self
            .get::<PaginationResponse<Vec<Action>>>(endpoint, Some(query_params))
            .await?;
        Ok(resp.content)
    }

    pub async fn get_target_attributes(
        &self,
        target_id: &str,
        filter_query: Option<&str>,
    ) -> HawkbitResult<HashMap<String, String>> {
        let endpoint = &format!("targets/{}/attributes", target_id);

        // Construct query parameters for pagination and filtering
        let mut query_params = HashMap::new();
        if let Some(filter) = filter_query {
            query_params.insert("q".to_string(), filter.to_string());
        }

        let response = self.get::<Value>(endpoint, Some(query_params)).await?;

        let mut attributes = HashMap::new();
        if let Some(map) = response.as_object() {
            for (key, value) in map {
                attributes.insert(
                    key.to_string(),
                    value.as_str().unwrap_or_default().to_string(),
                );
            }
        }

        Ok(attributes)
    }

    pub async fn get_action_detail(
        &self,
        target_id: &String,
        action_id: &i64,
    ) -> HawkbitResult<ActionDetail> {
        let endpoint = &format!("/targets/{}/actions/{}", target_id, action_id.to_string());

        self.get::<ActionDetail>(&endpoint, None).await
    }

    pub async fn cancel_action(
        &self,
        target_id: &String,
        action_id: &i64,
        force: bool,
    ) -> HawkbitResult<String> {
        let endpoint = &format!("/targets/{}/actions/{}", target_id, action_id.to_string());
        let mut query_params: HashMap<String, String> = HashMap::new();
        query_params.insert("force".to_string(), force.to_string());
        self.delete(&endpoint, Some(query_params)).await
    }

    pub async fn get_action_status(
        &self,
        target_id: &String,
        action_id: &i64,
    ) -> HawkbitResult<Vec<ActionStatusEvent>> {
        let endpoint = &format!(
            "/targets/{}/actions/{}/status",
            target_id,
            action_id.to_string()
        );

        let mut offset = 0;
        let mut total = usize::MAX; // Will be overwritten on first request
        let mut status = Vec::new();
        while offset < total {
            let mut query_params = HashMap::new();
            query_params.insert("sort".to_string(), "id:DESC".to_string());
            query_params.insert("representation".to_string(), "full".to_string());

            query_params.insert("offset".to_string(), offset.to_string());
            query_params.insert("limit".to_string(), 50.to_string());

            let new_page = self
                .get::<PaginationResponse<Vec<ActionStatusEvent>>>(&endpoint, Some(query_params))
                .await?;

            total = new_page.total;
            status.extend(new_page.content);
            if new_page.size == 0 {
                break;
            }

            offset += new_page.size;
        }
        Ok(status)
    }

    pub async fn assign_distribution(
        &self,
        target_id: &str,
        distribution_id: &u64,
    ) -> HawkbitResult<Value> {
        let endpoint = format!("targets/{}/assignedDS", target_id);
        let data = vec![json!({ "id": distribution_id, "type": "forced" })];
        self.post(&endpoint, &data).await
    }


    pub async fn get_distribution_sets(&self, filter_query: Option<&str>) -> HawkbitResult<Vec<DistributionSet>> {
        let mut offset = 0;
        let mut total = usize::MAX; // Will be overwritten on first request
        let mut distribution_sets = Vec::new();
        while offset < total {
            // Construct query parameters for pagination and filtering
            let mut query_params = HashMap::new();
            if let Some(filter) = filter_query {
                query_params.insert("q".to_string(), filter.to_string());
            }
            query_params.insert("offset".to_string(), offset.to_string());
            query_params.insert("limit".to_string(), 50.to_string());

            let new_page = self
                .get::<PaginationResponse<Vec<DistributionSet>>>("/distributionsets?sort=createdAt:DESC", Some(query_params))
                .await?;

            total = new_page.total;
            distribution_sets.extend(new_page.content);
            if new_page.size == 0 {
                break;
            }

            offset += new_page.size;
        }
        Ok(distribution_sets)
    }


    pub async fn get_distributionset(&self, distribution_id: &str) -> HawkbitResult<Value> {
        let mut query_params = HashMap::new();
        query_params.insert("sort".to_string(), "id:DESC".to_string());
        let endpoint = &format!("/distributionsets/{}", distribution_id.to_string());

        self.get::<Value>(&endpoint, Some(query_params)).await
    }

    pub async fn get_latest_distribution(&self) -> HawkbitResult<Value> {
        let v: Value = self
            .get("distributionsets?sort=createdAt:DESC&limit=1", None)
            .await?;
        if let Some(arr) = v.get("content").and_then(|c| c.as_array()) {
            if let Some(first) = arr.first() {
                return Ok(first.clone());
            }
        }
        Err(HawkbitError::new("No available distributions found"))
    }

    pub async fn update_target(
        &self,
        target_id: &str,
        target_name: &str,
        controller_id: &str,
        update: HashMap<String, String>,
    ) -> HawkbitResult<Option<Value>> {
        let mut body = update.clone();
        body.insert("name".to_string(), target_name.to_string());
        body.insert("controllerId".to_string(), controller_id.to_string());
        self.put(&format!("targets/{}", target_id), &json!(body))
            .await
    }

    pub async fn request_attributes(
        &self,
        target_id: &str,
        target_name: &str,
        controller_id: &str,
    ) -> HawkbitResult<Option<Value>> {
        let mut update = HashMap::new();
        update.insert("requestAttributes".to_string(), "true".to_string());
        self.update_target(target_id, target_name, controller_id, update)
            .await
    }
}
