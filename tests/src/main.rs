/// This file is generated by kopgen. Do not edit manually. If you need to make adjustments add it to .openapi-generator-ignore file.
pub mod utils;

fn main() {}

#[cfg(test)]
mod test {
    use crate::utils::{client, cluster, operator as operator_module};
    use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
    use kube::api::{Api, ObjectMeta};
    use operator::types::cat::{Cat, CatSpec};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_crds_exist() -> anyhow::Result<(), anyhow::Error> {
        cluster::setup().await?;
        operator_module::deploy().await?;
        let client = client::setup().await;

        let crds: Api<CustomResourceDefinition> = Api::all(client.clone());
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
        cluster::setup().await?;
        operator_module::deploy().await?;
        let client = client::setup().await;
        let api: Api<Cat> = Api::namespaced(client.clone(), "default");
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
            std::result::Result::Ok(_) => {}
            Err(_) => {
                api.create(&Default::default(), &resource).await?;
            }
        }

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
