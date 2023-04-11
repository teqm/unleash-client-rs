use crate::entity_tag::EntityTag;
use reqwest::{header, Client, RequestBuilder, StatusCode};
use std::time::Duration;
use unleash_types::client_features::{ClientFeatures, Query as FeaturesQuery};
use unleash_types::client_metrics::{ClientApplication, ClientMetrics};

pub struct UnleashClient {
    url: String,
    http_client: Client,
}

const UNLEASH_APPNAME_HEADER: &str = "UNLEASH-APPNAME";
const UNLEASH_INSTANCE_ID_HEADER: &str = "UNLEASH-INSTANCEID";
const UNLEASH_CLIENT_SPEC_HEADER: &str = "Unleash-Client-Spec";

pub enum ClientFeaturesResponse {
    NoUpdate(EntityTag),
    Updated(ClientFeatures, Option<EntityTag>),
}

impl UnleashClient {
    pub fn new(
        url: String,
        app_name: String,
        instance_id: Option<String>,
        token: String,
    ) -> UnleashClient {
        let mut header_map = header::HeaderMap::new();

        header_map.insert(
            UNLEASH_APPNAME_HEADER,
            header::HeaderValue::from_bytes(app_name.as_bytes()).unwrap(),
        );
        header_map.insert(
            UNLEASH_CLIENT_SPEC_HEADER,
            header::HeaderValue::from_static(unleash_yggdrasil::SUPPORTED_SPEC_VERSION),
        );
        header_map.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_bytes(token.as_bytes()).unwrap(),
        );

        if let Some(instance_id) = instance_id {
            header_map.insert(
                UNLEASH_INSTANCE_ID_HEADER,
                header::HeaderValue::from_bytes(instance_id.as_bytes()).unwrap(),
            );
        }

        let http_client = Client::builder()
            .default_headers(header_map)
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        UnleashClient { url, http_client }
    }

    fn client_features_req(
        &self,
        etag: Option<EntityTag>,
        query: Option<FeaturesQuery>,
    ) -> RequestBuilder {
        let mut req = self
            .http_client
            .get(format!("{}/api/client/features", self.url));

        if let Some(etag) = etag {
            req = req.header(header::IF_NONE_MATCH, etag.to_string());
        }

        if let Some(query) = query {
            req = req.query(&query);
        }

        req
    }

    pub async fn get_client_features(
        &self,
        etag: Option<EntityTag>,
        query: Option<FeaturesQuery>,
    ) -> Result<ClientFeaturesResponse, reqwest::Error> {
        let response = self
            .client_features_req(etag.clone(), query)
            .send()
            .await?
            .error_for_status()?;

        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(ClientFeaturesResponse::NoUpdate(
                etag.expect("etag should not be empty for `NOT_MODIFIED` response"),
            ));
        }

        let etag = response
            .headers()
            .get("ETag")
            .or_else(|| response.headers().get("etag"))
            .and_then(|etag| etag.to_str().unwrap().parse::<EntityTag>().ok());

        let features = response.json::<ClientFeatures>().await?;

        Ok(ClientFeaturesResponse::Updated(features, etag))
    }

    pub async fn register_as_client(
        &self,
        application: ClientApplication,
    ) -> Result<(), reqwest::Error> {
        self.http_client
            .post(format!("{}/api/client/register", self.url))
            .json(&application)
            .send()
            .await?;

        Ok(())
    }

    pub async fn send_metrics(&self, metrics: ClientMetrics) -> Result<(), reqwest::Error> {
        self.http_client
            .post(format!("{}/api/client/metrics", self.url))
            .json(&metrics)
            .send()
            .await?;

        Ok(())
    }
}
