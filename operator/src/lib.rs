pub mod cli;
pub mod controllers;
pub mod errors;
pub mod types;

use crate::errors::OperatorError;
use async_trait::async_trait;
use chrono::Utc;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, Time};
use kube::{
    api::{Api, Patch, PatchParams, Resource},
    core::object::HasStatus,
    Error,
};
use log::{debug, error, info, warn};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::fmt::Debug;
use tokio::time::{sleep, Duration};

pub const FINALIZER_DOMAIN: &str = "example.com";
pub const FIELD_MANAGER: &str = "operator";

#[async_trait]
pub trait KubeApi<T>: Send + Sync
where
    T: Resource + Clone + Send + Sync + 'static + DeserializeOwned + Serialize + Debug + HasStatus,
{
    async fn add_finalizer(&self, resource: &mut T) -> Result<(), OperatorError>;

    async fn remove_finalizer(&self, resource: &mut T) -> Result<(), OperatorError>;

    fn create_condition(
        &self,
        status: &str,
        type_: &str,
        reason: &str,
        message: &str,
        observed_generation: Option<i64>,
    ) -> Condition;

    async fn update_status(&self, status: &T) -> Result<(), OperatorError>;

    fn get_client(&self) -> Api<T>;

    fn set_client(&mut self, client: Api<T>);
}

pub struct KubeApiClient<T>
where
    T: Resource + Clone + Send + Sync + 'static + DeserializeOwned + Serialize + Debug + HasStatus,
    T::Status: Serialize,
{
    pub client: Api<T>,
}

#[async_trait]
impl<T> KubeApi<T> for KubeApiClient<T>
where
    T: Resource + Clone + Send + Sync + 'static + DeserializeOwned + Serialize + Debug + HasStatus,
    T::Status: Serialize,
{
    async fn add_finalizer(&self, resource: &mut T) -> Result<(), OperatorError> {
        let resource_name = resource.meta().name.clone().ok_or_else(|| {
            OperatorError::InvalidResourceState("Resource name is missing.".into())
        })?;

        let finalizer = format!("finalizers.{}", FINALIZER_DOMAIN);
        let finalizers = resource.meta_mut().finalizers.get_or_insert_with(Vec::new);

        if finalizers.contains(&finalizer) {
            debug!("Finalizer `{}` already exists for resource.", finalizer);
            return Ok(());
        }

        finalizers.push(finalizer.clone());

        let patch = Patch::Merge(json!({ "metadata": { "finalizers": finalizers } }));
        let patch_params = PatchParams::apply(FIELD_MANAGER);

        self.get_client()
            .patch(&resource_name, &patch_params, &patch)
            .await
            .map_err(|e| {
                error!(
                    "Failed to add finalizer `{}` to resource `{}`: {:?}",
                    finalizer, resource_name, e
                );
                OperatorError::FailedToPatchResource(e.into())
            })?;

        Ok(())
    }

    async fn remove_finalizer(&self, resource: &mut T) -> Result<(), OperatorError> {
        let resource_name = resource.meta().name.clone().ok_or_else(|| {
            OperatorError::InvalidResourceState("Resource name is missing.".into())
        })?;

        let finalizer = format!("finalizers.{}", FINALIZER_DOMAIN);
        let finalizers = match &mut resource.meta_mut().finalizers {
            Some(finalizers) => finalizers,
            None => return Ok(()),
        };

        if !finalizers.contains(&finalizer) {
            return Ok(());
        }

        finalizers.retain(|f| f != &finalizer);

        let patch = Patch::Merge(json!({ "metadata": { "finalizers": finalizers } }));
        let patch_params = PatchParams::apply(FIELD_MANAGER);

        self.get_client()
            .patch(&resource_name, &patch_params, &patch)
            .await
            .map_err(|e| {
                error!(
                    "Failed to remove finalizer from resource `{}`: {:?}",
                    resource_name, e
                );
                OperatorError::FailedToPatchResource(e.into())
            })?;

        Ok(())
    }

    fn create_condition(
        &self,
        status: &str,
        type_: &str,
        reason: &str,
        message: &str,
        observed_generation: Option<i64>,
    ) -> Condition {
        Condition {
            last_transition_time: Time(Utc::now()),
            message: message.to_string(),
            reason: reason.to_string(),
            status: status.to_string(),
            type_: type_.to_string(),
            observed_generation,
        }
    }

    async fn update_status(&self, status: &T) -> Result<(), OperatorError> {
        let resource_name = status.meta().name.clone().ok_or_else(|| {
            OperatorError::InvalidResourceState("Resource name is missing.".into())
        })?;

        let status_patch = if let Some(status) = status.status() {
            json!({ "status": status })
        } else {
            json!({ "status": null })
        };

        let patch = Patch::Merge(status_patch);
        let patch_params = PatchParams::apply(FIELD_MANAGER);

        for _ in 0..3 {
            match self
                .get_client()
                .patch_status(&resource_name, &patch_params, &patch)
                .await
            {
                Ok(_) => {
                    info!(
                        "Successfully updated status for resource `{}`.",
                        resource_name
                    );
                    return Ok(());
                }
                Err(Error::Api(ae)) if ae.code == 409 => {
                    warn!(
                        "Conflict updating status for `{}`, retrying...",
                        resource_name
                    );
                    sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Failed to update status for `{}`: {:?}", resource_name, e);
                    return Err(OperatorError::FailedToUpdateStatus(e.into()));
                }
            }
        }

        Err(OperatorError::FailedToUpdateStatus(anyhow::anyhow!(
            "Failed to update status after retries."
        )))
    }

    fn get_client(&self) -> Api<T> {
        self.client.clone()
    }

    fn set_client(&mut self, client: Api<T>) {
        self.client = client;
    }
}
