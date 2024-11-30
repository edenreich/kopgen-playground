use std::sync::Arc;

// This file is generated by kopgen. Do not edit manually. If you need to make adjustments add it to .openapi-generator-ignore file.
use anyhow::Context;
use clap::Parser;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{api::Api, Client as KubeClient, CustomResourceExt};
use log::{debug, error, info};
use openapi::apis::configuration::Configuration;
use operator::{
    cli::{Cli, Commands},
    deploy_crd, wait_for_crd,
};
use warp::Filter;

use operator::{
    controllers::cats,
    types::{cat::Cat, dog::Dog, horse::Horse},
};

use openapi::apis::cats_api::CatsApiClient;

const API_URL: &str = "http://localhost:8080";
const API_USER_AGENT: &str = "k8s-operator";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    debug!("Log level: {}", cli.verbosity);

    match cli.command {
        Some(Commands::Run { install_crds }) => {
            info!("Starting operator...");
            debug!("CRD's will be installed automatically: {}", install_crds);

            let kube_client = KubeClient::try_default().await?;
            let kube_client_api: Api<CustomResourceDefinition> = Api::all(kube_client.clone());

            if install_crds {
                info!("Deploying CRDs...");

                let crds = vec![Cat::crd(), Dog::crd(), Horse::crd()];

                for crd in crds {
                    deploy_crd(kube_client_api.clone(), crd).await?;
                }
            }

            let controllers_crds = vec![format!("cats.example.com")];
            for controller_crd in controllers_crds {
                if let Err(e) = wait_for_crd(kube_client_api.clone(), &controller_crd).await {
                    error!("Error waiting for CRD {}: {}", &controller_crd, e);
                }
            }

            let config = Arc::new(Configuration {
                base_path: API_URL.to_string(),
                client: reqwest::Client::new(),
                user_agent: Some(API_USER_AGENT.to_string()),
                bearer_access_token: Some(std::env::var("ACCESS_TOKEN").unwrap_or_default()),
                ..Default::default()
            });

            // Start the cats controller for the cats.example.com/v1 API group
            let kube_client = Api::namespaced(kube_client.clone(), "default");
            let cats_client = Arc::new(CatsApiClient::new(config));
            tokio::spawn(async {
                let _cats_controller = cats::handle(kube_client, cats_client).await;
            });

            tokio::spawn(async {
                let liveness_route = warp::path!("healthz")
                    .map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));

                let readiness_route = warp::path!("readyz")
                    .map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));

                let health_routes = liveness_route.or(readiness_route);

                warp::serve(health_routes).run(([0, 0, 0, 0], 8000)).await;
            });
        }
        Some(Commands::Version) => {
            println!("Operator version: {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        None => {
            error!("No command provided. Use --help for more information.");
            return Err(anyhow::anyhow!(
                "No command provided. Use --help for more information."
            ));
        }
    }

    tokio::signal::ctrl_c()
        .await
        .context("Failed to listen for Ctrl+C")?;
    info!("Termination signal received. Shutting down.");

    Ok(())
}
