// This file is generated by kopgen. Do not edit manually. If you need to make adjustments add it to .openapi-generator-ignore file.
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;

/// Represents a fake server
#[derive(Default)]
pub struct FakeServer {}

impl FakeServer {
    /// Creates a new instance of `FakeServer`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fake_server::FakeServer;
    ///
    /// let fake_server = FakeServer::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Packages the fake-server Docker image and pushes it to the specified container registry.
    ///
    /// This method builds the Docker image using the provided container registry and tags it as `fake-server:latest`.
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
    /// use fake_server::FakeServer;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let fake_server = FakeServer::new();
    ///     fake_server.package("localhost:5005").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn package(&self, container_registry: &str) -> Result<()> {
        Command::new("docker")
            .args([
                "build",
                "-t",
                &format!("{}/fake-server:latest", container_registry),
                "-f",
                "fake-server/Dockerfile",
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
            .args([
                "push",
                &format!("{}/fake-server:latest", container_registry),
            ])
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

    /// Deploys the fake-server to the specified Kubernetes cluster.
    ///
    /// This method switches the kubectl context to the given cluster, applies the fake-server manifests,
    /// and waits for the fake-server deployment to roll out successfully.
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
    /// use fake_server::FakeServer;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let fake_server = FakeServer::new();
    ///     fake_server.deploy_on("k3d-k3s-default").await?;
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
            .args(["apply", "-f", "fake-server/deployment.yaml"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl apply` for fake-server")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl apply` for fake-server failed"))?;

        Command::new("kubectl")
            .args([
                "rollout",
                "status",
                "deployment/fake-server",
                "--timeout=60s",
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl rollout status` for fake-server")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl rollout status` for fake-server failed"))?;

        Ok(())
    }

    /// Undeploys the fake-server from the specified Kubernetes cluster.
    ///
    /// This method switches the kubectl context to the given cluster and deletes the fake-server deployment.
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
    /// use fake_server::FakeServer;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let fake_server = FakeServer::new();
    ///     fake_server.undeploy_from("k3d-cluster").await?;
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
            .args(["delete", "-f", "fake-server/deployment.yaml", "--wait"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl delete` for fake-server")?;

        Ok(())
    }
}
