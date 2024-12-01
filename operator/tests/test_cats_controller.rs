#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, Time};
    use kube::Api;
    use mockall::mock;
    use openapi::apis::cats_api::{
        CreateCatError, DeleteCatByIdError, GetCatByIdError, GetCatsError, UpdateCatByIdError,
    };
    use openapi::apis::ResponseContent;
    use openapi::{apis::cats_api::CatsApi, apis::Error, models::Cat as CatDto};
    use operator::{
        controllers::cats::{converters, handle_create},
        errors::OperatorError,
        types::cat::{Cat, CatSpec},
        KubeApi,
    };
    use std::sync::Arc;
    use uuid::Uuid;

    mock! {
        pub CatsApi {}

        #[async_trait]
        impl CatsApi for CatsApi {
            async fn create_cat<'cat>(&self, cat: CatDto) -> Result<CatDto, Error<CreateCatError>>;
            async fn delete_cat_by_id<'id>(&self, id: &'id str) -> Result<(), Error<DeleteCatByIdError>>;
            async fn get_cat_by_id<'id>(&self, id: &'id str) -> Result<CatDto, Error<GetCatByIdError>>;
            async fn update_cat_by_id<'id, 'cat>(&self, id: &'id str, cat: CatDto) -> Result<CatDto, Error<UpdateCatByIdError>>;
            async fn get_cats(&self) -> Result<Vec<CatDto>, Error<GetCatsError>>;
        }
    }

    mock! {
        pub KubeApiClient {}

        #[async_trait]
        impl KubeApi<Cat> for KubeApiClient {
            async fn add_finalizer(&self, resource: &mut Cat) -> Result<(), OperatorError>;
            async fn remove_finalizer(&self, resource: &mut Cat) -> Result<(), OperatorError>;
            fn create_condition(
                &self,
                status: &str,
                type_: &str,
                reason: &str,
                message: &str,
                observed_generation: Option<i64>,
            ) -> Condition;
            async fn update_status(&self, status: &Cat) -> Result<(), OperatorError>;
            fn get_client(&self) -> Api<Cat>;
            fn set_client(&mut self, client: Api<Cat>);
        }
    }

    #[tokio::test]
    async fn test_handle_create_success() {
        let mut kube_client = MockKubeApiClient::new();
        let mut mock_cats_api = MockCatsApi::new();

        let mut cat = Cat {
            metadata: kube::api::ObjectMeta {
                name: Some("whiskers".to_string()),
                ..Default::default()
            },
            spec: CatSpec {
                name: "Whiskers".to_string(),
                breed: "Siamese".to_string(),
                age: 3,
            },
            status: None,
        };

        let remote_cat = CatDto {
            uuid: Some(Uuid::new_v4()),
            name: cat.spec.name.clone(),
            breed: cat.spec.breed.clone(),
            age: cat.spec.age,
        };

        mock_cats_api
            .expect_create_cat()
            .withf(move |dto| {
                let kube_spec = CatSpec {
                    name: dto.name.clone(),
                    breed: dto.breed.clone(),
                    age: dto.age,
                };

                dto.name == kube_spec.name
                    && dto.breed == kube_spec.breed
                    && dto.age == kube_spec.age
            })
            .times(1)
            .returning(move |dto| {
                Ok(CatDto {
                    uuid: remote_cat.uuid,
                    ..dto
                })
            });

        let cats_api = Arc::new(mock_cats_api) as Arc<dyn CatsApi>;

        kube_client
            .expect_update_status()
            .times(1)
            .returning(|_| Ok(()));

        kube_client
            .expect_add_finalizer()
            .times(1)
            .returning(|_| Ok(()));

        kube_client
            .expect_create_condition()
            .times(1)
            .returning(|_, _, _, _, _| Condition {
                last_transition_time: Time(chrono::Utc::now()),
                message: "The cat has been created successfully".to_string(),
                observed_generation: Some(0),
                reason: "CatCreated".to_string(),
                status: "True".to_string(),
                type_: "AvailableCreated".to_string(),
            });

        let kube_client = Arc::new(kube_client) as Arc<dyn KubeApi<Cat>>;

        let result = handle_create(kube_client.as_ref(), cats_api.as_ref(), &mut cat).await;

        assert!(result.is_ok());
        assert!(cat.status.is_some());

        let status = cat.status.as_ref().unwrap();
        assert_eq!(status.uuid, converters::uuid_to_string(remote_cat.uuid));
        // assert_eq!(status.observed_generation, Some(0)); // TODO - Fix this
        assert_eq!(status.conditions.len(), 1);

        let condition = &status.conditions[0];
        assert_eq!(condition.type_, "AvailableCreated");
        assert_eq!(condition.status, "True");
        assert_eq!(condition.reason, "CatCreated");
        assert_eq!(condition.message, "The cat has been created successfully");
        assert!(condition.last_transition_time.0.timestamp() > 0);
    }

    #[tokio::test]
    async fn test_handle_create_failed() {
        let mut kube_client = MockKubeApiClient::new();
        let mut mock_cats_api = MockCatsApi::new();

        let mut cat = Cat {
            metadata: kube::api::ObjectMeta {
                name: Some("whiskers".to_string()),
                ..Default::default()
            },
            spec: CatSpec {
                name: "Whiskers".to_string(),
                breed: "Siamese".to_string(),
                age: 3,
            },
            status: None,
        };

        mock_cats_api.expect_create_cat().times(1).returning(|_| {
            Err(Error::ResponseError(ResponseContent {
                status: reqwest::StatusCode::BAD_REQUEST,
                content: "Internal Server Error".to_string(),
                entity: None,
            }))
        });

        kube_client
            .expect_create_condition()
            .times(1)
            .returning(|_, _, _, _, _| Condition {
                last_transition_time: Time(chrono::Utc::now()),
                message: "Failed to create the cat".to_string(),
                observed_generation: Some(0),
                reason: "CatNotCreated".to_string(),
                status: "False".to_string(),
                type_: "AvailableNotCreated".to_string(),
            });

        kube_client
            .expect_update_status()
            .times(1)
            .returning(|_| Ok(()));

        let cats_api = Arc::new(mock_cats_api) as Arc<dyn CatsApi>;

        let result = handle_create(&kube_client, cats_api.as_ref(), &mut cat).await;

        assert!(result.is_err());
        assert!(cat.status.is_some());
        let status = cat.status.as_ref().unwrap();
        assert!(status.uuid.is_none());
        assert_eq!(status.conditions.len(), 1);
    }
}
