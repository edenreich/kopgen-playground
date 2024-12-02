use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;

pub struct FakeServer {
    container_registry: String,
    cluster_name: String,
}

impl FakeServer {
    pub fn new(container_registry: String, cluster_name: String) -> Self {
        Self {
            container_registry,
            cluster_name,
        }
    }

    /// Packages the fake-server Docker image.
    pub async fn package(&self) -> Result<()> {
        // Build the Docker image
        Command::new("docker")
            .args([
                "build",
                "-t",
                &format!("{}/fake-server:latest", self.container_registry),
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

        // Push the Docker image
        Command::new("docker")
            .args([
                "push",
                &format!("{}/fake-server:latest", self.container_registry),
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

    /// Deploys the fake-server to the Kubernetes cluster.
    pub async fn deploy(&self) -> Result<()> {
        // Set the Kubernetes context
        Command::new("kubectl")
            .args(["config", "use-context", &self.cluster_name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl config use-context`")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl config use-context` failed"))?;

        // Apply the fake-server manifests
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

        // Wait for the Deployment to be ready
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

    /// Tears down the fake-server from the Kubernetes cluster.
    pub async fn teardown(&self) -> Result<()> {
        // Set the Kubernetes context
        Command::new("kubectl")
            .args(["config", "use-context", &self.cluster_name])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl config use-context` for teardown")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("`kubectl config use-context` for teardown failed"))?;

        // Delete the fake-server manifests
        Command::new("kubectl")
            .args(["delete", "-f", "fake-server/deployment.yaml"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl delete` for fake-server")?;

        Ok(())
    }
}
