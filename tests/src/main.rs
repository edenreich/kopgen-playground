/// This file is generated by kopgen. Do not edit manually. If you need to make adjustments add it to .openapi-generator-ignore file.
pub mod utils;

fn main() {
    env_logger::init();
}

#[cfg(test)]
mod test {
    use crate::utils::{
        client::{self, Waiter},
        cluster,
        fake_server::FakeServer,
        operator as operator_module,
    };
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
    use kube::api::{Api, ObjectMeta};
    use operator::types::cat::{Cat, CatSpec};
    use serial_test::serial;
    use std::{result::Result, time::Duration};

    #[tokio::test]
    #[serial]
    async fn test_crds_exist() -> anyhow::Result<(), anyhow::Error> {
        let _ = cluster::setup().await?;
        operator_module::deploy().await?;

        let crds: Api<CustomResourceDefinition> = client::setup_crd().await?;
        let params = kube::api::ListParams {
            field_selector: Some("metadata.name=cats.example.com".to_string()),
            ..Default::default()
        };
        let crds_list = crds.list(&params).await?;

        cluster::teardown().await?;

        assert_eq!(
            crds_list.items.len(),
            1,
            "CRDs for cats.example.com not found"
        );

        anyhow::Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_add_finalizer() -> anyhow::Result<(), anyhow::Error> {
        let cluster = cluster::setup().await?;

        let fake_server = FakeServer::new();
        fake_server.package("localhost:5005").await?;
        fake_server.deploy_on(&cluster).await?;

        operator_module::deploy().await?;

        let api: Api<Cat> = client::setup().await?;
        let resource = Cat {
            metadata: ObjectMeta {
                name: Some("test-cat".to_string()),
                ..Default::default()
            },
            spec: CatSpec {
                name: "test-cat".to_string(),
                age: 1,
                breed: "test".to_string(),
            },
            status: None,
        };

        // apply the resource
        match api.get("test-cat").await {
            Result::Ok(_) => {}
            Err(_) => {
                api.create(&Default::default(), &resource).await?;
            }
        }

        // wait for the resource to be created
        api.wait_for_field("test-cat", "$.metadata.finalizers", Duration::from_secs(30))
            .await?;

        // get the resource
        let cat = api.get("test-cat").await?;

        // check if the finalizer is added
        assert_eq!(
            cat.metadata.finalizers,
            Some(vec!["finalizers.example.com".to_string()])
        );

        cluster::teardown().await?;

        anyhow::Ok(())
    }
}
