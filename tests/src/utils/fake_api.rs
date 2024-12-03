// This file is generated by kopgen. Do not edit manually. If you need to make adjustments add it to .openapi-generator-ignore file.
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;

/// Represents a fake server
#[derive(Default)]
pub struct FakeApi {}

impl FakeApi {
    /// Creates a new instance of `FakeApi`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fake_api::FakeApi;
    ///
    /// let fake_api = FakeApi::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Packages the fake-api Docker image and pushes it to the specified container registry.
    ///
    /// This method builds the Docker image using the provided container registry and tags it as `fake-api:latest`.
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
    /// use fake_api::FakeApi;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let fake_api = FakeApi::new();
    ///     fake_api.package("localhost:5005").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn package(&self, container_registry: &str) -> Result<()> {
        Command::new("docker")
            .args([
                "build",
                "-t",
                &format!("{}/fake-api:latest", container_registry),
                "-f",
                "fake-api/Dockerfile",
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
            .args(["push", &format!("{}/fake-api:latest", container_registry)])
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

    /// Deploys the fake-api to the specified Kubernetes cluster.
    ///
    /// This method switches the kubectl context to the given cluster, applies the fake-api manifests,
    /// and waits for the fake-api deployment to roll out successfully.
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
    /// use fake_api::FakeApi;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let fake_api = FakeApi::new();
    ///     fake_api.deploy_on("k3d-k3s-default").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn deploy_on(&self, cluster_name: &str) -> Result<()> {
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
            .args(["apply", "-f", "fake-api/deployment.yaml"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl apply` for fake-api")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl apply` for fake-api failed"))?;

        Command::new("kubectl")
            .args(["rollout", "status", "deployment/fake-api", "--timeout=60s"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl rollout status` for fake-api")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl rollout status` for fake-api failed"))?;

        Ok(())
    }

    /// Undeploys the fake-api from the specified Kubernetes cluster.
    ///
    /// This method switches the kubectl context to the given cluster and deletes the fake-api deployment.
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
    /// use fake_api::FakeApi;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let fake_api = FakeApi::new();
    ///     fake_api.undeploy_from("k3d-cluster").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn undeploy_from(&self, cluster_name: &str) -> Result<()> {
        Command::new("kubectl")
            .args(["config", "use-context", cluster_name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl config use-context` for teardown")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl config use-context` for teardown failed"))?;

        Command::new("kubectl")
            .args(["delete", "-f", "fake-api/deployment.yaml"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl delete` for fake-api")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl config use-context` for teardown failed"))?;

        Ok(())
    }
}
