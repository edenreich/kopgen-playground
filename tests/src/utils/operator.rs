// This file is generated by kopgen. Do not edit manually. If you need to make adjustments add it to .openapi-generator-ignore file.
use anyhow::Context;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{api::PostParams, Api};
use std::process::Stdio;
use tokio::process::Command;

/// Represents the Kubernetes Operator
#[derive(Default)]
pub struct Operator {
    config: Option<ConfigMap>,
}

impl Operator {
    /// Creates a new instance of the `Operator`.
    ///
    /// # Examples
    ///
    /// ```
    /// use operator::Operator;
    ///
    /// let operator = Operator::new();
    /// ```
    pub fn new(config: Option<ConfigMap>) -> Self {
        Self { config }
    }

    /// Packages the operator Docker image and pushes it to the specified container registry.
    ///
    /// This method builds the Docker image using the provided container registry and tags it as `operator:latest`.
    /// After building, it pushes the image to the local container registry.
    ///
    /// # Arguments
    ///
    /// * `container_registry` - A string slice that holds the address of the local container registry.
    ///
    /// # Errors
    ///
    /// Returns an error if the Docker build or push commands fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use operator::Operator;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let operator = Operator::new();
    ///     operator.package("localhost:5005").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn package(&self, container_registry: &str) -> anyhow::Result<()> {
        Command::new("docker")
            .args([
                "build",
                "-t",
                &format!("{}/operator:latest", container_registry),
                "-f",
                "../Dockerfile",
                "..",
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `docker build`")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`docker build` failed"))?;

        Command::new("docker")
            .args(["push", &format!("{}/operator:latest", container_registry)])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `docker push`")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`docker push` failed"))?;

        Ok(())
    }

    /// Deploys the operator to the specified Kubernetes cluster.
    ///
    /// This method switches the kubectl context to the given cluster, applies the RBAC and operator manifests,
    /// and waits for the operator deployment to roll out successfully.
    ///
    /// # Arguments
    ///
    /// * `cluster_name` - A string slice that holds the name of the Kubernetes cluster.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the kubectl commands fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use operator::Operator;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let operator = Operator::new();
    ///     operator.deploy_on("k3d-cluster").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn deploy_on(&self, cluster_name: &str) -> anyhow::Result<()> {
        Command::new("kubectl")
            .args(["config", "use-context", cluster_name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl config use-context`")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl config use-context` failed"))?;

        Command::new("kubectl")
            .args(["apply", "-f", "../manifests/rbac/"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl apply` for RBAC")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl apply` for RBAC failed"))?;

        Command::new("kubectl")
            .args(["apply", "-f", "../manifests/operator/configmap.yaml"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl apply` for operator configmap")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl apply` for operator configmap failed"))?;

        if let Some(config) = &self.config {
            let client = kube::Client::try_default().await?;
            let cms = Api::<ConfigMap>::namespaced(client.clone(), "default");
            let _ = cms.delete("operator-config", &Default::default()).await;
            cms.create(&PostParams::default(), config).await?;
        }

        Command::new("kubectl")
            .args([
                "apply",
                "-f",
                "../manifests/operator/secret.yaml",
                "-f",
                "../manifests/operator/deployment.yaml",
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl apply` for operator deployment and secret")?
            .success()
            .then_some(())
            .ok_or_else(|| {
                anyhow::anyhow!("`kubectl apply` for operator deployment and secret failed")
            })?;

        Command::new("kubectl")
            .args(["rollout", "status", "deployment/operator", "--timeout=60s"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl rollout status` for operator")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl rollout status` for operator failed"))?;

        Ok(())
    }

    /// Undeploys the operator from the specified Kubernetes cluster.
    ///
    /// This method switches the kubectl context to the given cluster and deletes the operator deployment.
    ///
    /// # Arguments
    ///
    /// * `cluster_name` - A string slice that holds the name of the Kubernetes cluster.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the kubectl commands fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use operator::Operator;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let operator = Operator::new();
    ///     operator.undeploy_from("k3d-cluster").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn undeploy_from(&self, cluster_name: &str) -> anyhow::Result<()> {
        Command::new("kubectl")
            .args(["config", "use-context", cluster_name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl config use-context`")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl config use-context` failed"))?;

        Command::new("kubectl")
            .args(["delete", "-f", "../manifests/operator/"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl delete`")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl delete` failed"))?;

        Command::new("kubectl")
            .args(["delete", "-f", "../manifests/rbac/"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl delete`")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl delete` failed"))?;

        Ok(())
    }
}
