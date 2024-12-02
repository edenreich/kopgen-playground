use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;

#[derive(Default)]
pub struct FakeServer {}

impl FakeServer {
    pub fn new() -> Self {
        Self::default()
    }

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
            .args(["delete", "-f", "fake-server/deployment.yaml"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("Failed to execute `kubectl delete` for fake-server")?;

        Ok(())
    }
}
