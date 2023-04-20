use crate::entity_tag::EntityTag;
use crate::token::UnleashToken;
use crate::unleash_client::{ClientFeaturesResponse, UnleashClient};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use std::time::Duration;
use tracing::{debug, warn};
use unleash_types::client_features::{Query as FeaturesQuery};
use unleash_types::client_metrics::{ClientApplication, ClientMetrics, MetricBucket};
use unleash_yggdrasil::{EngineState, ResolvedToggle, VariantDef, Context};

pub struct Unleash {
    app_name: String,
    instance_id: Option<String>,
    environment: String,
    refresh_interval: Duration,
    features_query: Option<FeaturesQuery>,
    unleash_client: UnleashClient,
    etag: RwLock<Option<EntityTag>>,
    engine_state: RwLock<EngineState>,
    enabled: AtomicBool,
}

impl Unleash {
    pub fn new(
        url: String,
        app_name: String,
        token: String,
        instance_id: Option<String>,
        refresh_interval: Option<Duration>,
        features_query: Option<FeaturesQuery>,
    ) -> Unleash {
        let unleash_token = UnleashToken::try_from(token).unwrap();

        let unleash_client = UnleashClient::new(
            url,
            app_name.clone(),
            instance_id.clone(),
            unleash_token.token,
        );

        Unleash {
            app_name,
            instance_id,
            environment: unleash_token.environment,
            features_query,
            refresh_interval: refresh_interval.unwrap_or(Duration::from_secs(15)),
            unleash_client,
            etag: RwLock::new(None),
            engine_state: RwLock::new(EngineState::default()),
            enabled: AtomicBool::new(false),
        }
    }

    pub fn is_enabled(&self, name: &str, context: &Context) -> bool {
        let enhanced_context = Context {
            app_name: Some(self.app_name.clone()),
            environment: Some(self.environment.clone()),
            ..context.clone()
        };

        self.engine_state
            .read()
            .unwrap()
            .is_enabled(name, &enhanced_context)
    }

    pub fn get_variant(&self, name: &str, context: &Context) -> VariantDef {
        let enhanced_context = Context {
            app_name: Some(self.app_name.clone()),
            environment: Some(self.environment.clone()),
            ..context.clone()
        };

        self.engine_state
            .read()
            .unwrap()
            .get_variant(name, &enhanced_context)
    }

    pub fn resolve_all(&self, context: &Context) -> Option<HashMap<String, ResolvedToggle>> {
        let enhanced_context = Context {
            app_name: Some(self.app_name.clone()),
            environment: Some(self.environment.clone()),
            ..context.clone()
        };

        self.engine_state
            .read()
            .unwrap()
            .resolve_all(&enhanced_context)
    }

    pub async fn start(&self) {
        self.enabled.store(true, Ordering::Relaxed);

        self.register().await;

        while self.enabled.load(Ordering::Relaxed) {
            let metrics_bucket = self.refresh().await;

            self.send_metrics(metrics_bucket).await;

            tokio::time::sleep(self.refresh_interval).await;
        }
    }

    pub fn stop(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    async fn register(&self) {
        let register_result = self
            .unleash_client
            .register_as_client(ClientApplication {
                app_name: self.app_name.clone(),
                connect_via: None,
                environment: Some(self.environment.clone()),
                instance_id: self.instance_id.clone(),
                interval: self.refresh_interval.as_secs() as u32,
                sdk_version: Some(format!("unleash-client-rs:{}", env!("CARGO_PKG_VERSION"))),
                started: Utc::now(),
                strategies: vec![],
            })
            .await;

        if let Err(e) = register_result {
            warn!("{}", e);
        }
    }

    async fn send_metrics(&self, metrics_bucket: Option<MetricBucket>) {
        if let Some(bucket) = metrics_bucket {
            let metrics_result = self
                .unleash_client
                .send_metrics(ClientMetrics {
                    app_name: self.app_name.clone(),
                    instance_id: self.instance_id.clone(),
                    environment: Some(self.environment.clone()),
                    bucket,
                })
                .await;

            if let Err(e) = metrics_result {
                warn!("{}", e);
            }
        }
    }

    async fn refresh(&self) -> Option<MetricBucket> {
        let etag = self.etag.read().unwrap().clone();

        let features_result = self
            .unleash_client
            .get_client_features(etag, self.features_query.clone())
            .await;

        let metrics_bucket = match features_result {
            Ok(feature_response) => match feature_response {
                ClientFeaturesResponse::NoUpdate(etag) => {
                    debug!("no update needed, will update with {etag}");

                    self.engine_state.write().unwrap().get_metrics()
                }
                ClientFeaturesResponse::Updated(features, etag) => {
                    debug!("got updated client features, updating features with {etag:?}");

                    let mut e = self.etag.write().unwrap();
                    *e = etag;

                    self.engine_state.write().unwrap().take_state(features)
                }
            },
            Err(e) => {
                warn!("{}", e);

                None
            }
        };

        metrics_bucket
    }
}

#[derive(Default)]
pub struct UnleashBuilder {
    instance_id: Option<String>,
    refresh_interval: Option<Duration>,
    features_query: Option<FeaturesQuery>,
}

impl UnleashBuilder {
    pub fn instance_id(mut self, instance_id: String) -> Self {
        self.instance_id = Some(instance_id);
        self
    }

    pub fn refresh_interval(mut self, refresh_interval: Duration) -> Self {
        self.refresh_interval = Some(refresh_interval);
        self
    }

    pub fn features_query(mut self, features_query: FeaturesQuery) -> Self {
        self.features_query = Some(features_query);
        self
    }

    pub fn build(self, url: String, app_name: String, token: String) -> Unleash {
        Unleash::new(
            url,
            app_name,
            token,
            self.instance_id,
            self.refresh_interval,
            self.features_query,
        )
    }
}
