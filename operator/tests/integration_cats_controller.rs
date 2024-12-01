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
        controllers::cats::{converters, handle_create, reconcile, ContextData},
        errors::OperatorError,
        types::cat::{Cat, CatSpec, CatStatus},
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
            fn create_condition(&self, status: &str, type_: &str, reason: &str, message: &str, observed_generation: Option<i64>) -> Condition;
            async fn update_status(&self, status: &Cat) -> Result<(), OperatorError>;
            async fn replace(&self, name: &str, post_params: &kube::api::PostParams, resource: &Cat) -> Result<Cat, OperatorError>;
            fn get_client(&self) -> Api<Cat>;
            fn set_client(&mut self, client: Api<Cat>);
        }
    }

    fn setup_cat() -> Cat {
        Cat {
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
        }
    }

    #[tokio::test]
    async fn test_handle_create_success() {
        let mut kube_client = MockKubeApiClient::new();
        let mut mock_cats_api = MockCatsApi::new();
        let mut cat = setup_cat();

        let remote_cat = CatDto {
            uuid: Some(Uuid::new_v4()),
            name: cat.spec.name.clone(),
            breed: cat.spec.breed.clone(),
            age: cat.spec.age,
        };

        let expected_name = cat.spec.name.clone();
        let expected_breed = cat.spec.breed.clone();
        let expected_age = cat.spec.age;

        mock_cats_api
            .expect_create_cat()
            .withf(move |dto| {
                dto.name == expected_name && dto.breed == expected_breed && dto.age == expected_age
            })
            .times(1)
            .returning(move |dto| {
                Ok(CatDto {
                    uuid: remote_cat.uuid,
                    ..dto
                })
            });

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

        let cats_api = Arc::new(mock_cats_api) as Arc<dyn CatsApi>;
        let kube_client = Arc::new(kube_client) as Arc<dyn KubeApi<Cat>>;

        // Now it's safe to borrow `cat` mutably without partial moves
        let result = handle_create(kube_client.as_ref(), cats_api.as_ref(), &mut cat).await;

        assert!(result.is_ok());
        assert!(cat.status.is_some());

        let status = cat.status.as_ref().unwrap();
        assert_eq!(status.uuid, converters::uuid_to_string(remote_cat.uuid));
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
        let mut cat = setup_cat();

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
                status: "False".to_string(),
                type_: "AvailableNotCreated".to_string(),
                reason: "CatNotCreated".to_string(),
                message: "Failed to create the cat".to_string(),
                observed_generation: Some(0),
                last_transition_time: Time(chrono::Utc::now()),
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

    #[tokio::test]
    async fn test_reconcile_new_resource() {
        let mut kube_client = MockKubeApiClient::new();
        let mut mock_cats_api = MockCatsApi::new();
        let cat = Arc::new(setup_cat());

        let remote_cat = CatDto {
            uuid: Some(Uuid::new_v4()),
            name: cat.spec.name.clone(),
            breed: cat.spec.breed.clone(),
            age: cat.spec.age,
        };

        let remote_cat_clone = remote_cat.clone();

        let expected_name = cat.spec.name.clone();
        let expected_breed = cat.spec.breed.clone();
        let expected_age = cat.spec.age;

        mock_cats_api
            .expect_get_cat_by_id()
            .times(1)
            .returning(move |_| Ok(remote_cat.clone()));

        mock_cats_api
            .expect_create_cat()
            .withf(move |dto| {
                dto.name == expected_name && dto.breed == expected_breed && dto.age == expected_age
            })
            .times(1)
            .returning(move |dto| {
                Ok(CatDto {
                    uuid: remote_cat_clone.uuid,
                    ..dto
                })
            });

        kube_client
            .expect_add_finalizer()
            .times(1)
            .returning(|_| Ok(()));

        kube_client
            .expect_create_condition()
            .times(1)
            .withf(|status, type_, reason, message, observed_generation| {
                status == "Created"
                    && type_ == "AvailableCreated"
                    && reason == "Created the resource"
                    && message == "Resource has been created"
                    && *observed_generation == None
            })
            .returning(
                |status, type_, reason, message, observed_generation| Condition {
                    status: status.to_string(),
                    type_: type_.to_string(),
                    reason: reason.to_string(),
                    message: message.to_string(),
                    observed_generation: observed_generation,
                    last_transition_time: Time(chrono::Utc::now()),
                },
            );

        kube_client
            .expect_update_status()
            .times(1)
            .returning(|_| Ok(()));

        let kube_client = Arc::new(kube_client) as Arc<dyn KubeApi<Cat>>;
        let cats_api = Arc::new(mock_cats_api) as Arc<dyn CatsApi>;

        let result = reconcile(
            Arc::clone(&cat),
            Arc::new(ContextData {
                kube_client: kube_client.clone(),
                cats_client: cats_api.clone(),
            }),
        )
        .await;

        assert!(result.is_ok());
        println!("{:#?}", cat);
        assert!(cat.status.is_some());
    }

    #[tokio::test]
    async fn test_reconcile_failed_to_create_resource_because_of_internal_server_error() {
        let mut kube_client = MockKubeApiClient::new();
        let mut mock_cats_api = MockCatsApi::new();
        let cat = Arc::new(setup_cat());

        mock_cats_api.expect_get_cat_by_id().times(0);
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
                status: "Failed".to_string(),
                type_: "AvailableFailed".to_string(),
                reason: "CatNotCreated".to_string(),
                message: "Resource has not been created".to_string(),
                observed_generation: Some(0),
                last_transition_time: Time(chrono::Utc::now()),
            });

        kube_client
            .expect_update_status()
            .times(1)
            .returning(|_| Ok(()));

        let cats_api = Arc::new(mock_cats_api) as Arc<dyn CatsApi>;
        let kube_client = Arc::new(kube_client) as Arc<dyn KubeApi<Cat>>;

        let result = reconcile(
            Arc::clone(&cat),
            Arc::new(ContextData {
                kube_client: kube_client.clone(),
                cats_client: cats_api.clone(),
            }),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reconcile_not_executing_creation_nor_update_if_observation_generation_and_meta_observation_equal(
    ) {
        let mut kube_client = MockKubeApiClient::new();
        let mut mock_cats_api = MockCatsApi::new();
        let uuid = Uuid::new_v4();

        let cat = Arc::new(Cat {
            metadata: kube::api::ObjectMeta {
                name: Some("whiskers".to_string()),
                generation: Some(1),
                ..Default::default()
            },
            spec: CatSpec {
                name: "Whiskers".to_string(),
                breed: "Siamese".to_string(),
                age: 3,
            },
            status: Some(CatStatus {
                uuid: Some(uuid.to_string()),
                observed_generation: Some(1),
                conditions: vec![],
            }),
        });

        let remote_cat = CatDto {
            uuid: Some(uuid),
            name: cat.spec.name.clone(),
            breed: cat.spec.breed.clone(),
            age: cat.spec.age,
        };

        mock_cats_api
            .expect_get_cat_by_id()
            .times(1)
            .returning(move |_| Ok(remote_cat.clone()));
        mock_cats_api.expect_create_cat().times(0);
        mock_cats_api.expect_update_cat_by_id().times(0);
        kube_client.expect_create_condition().times(0);
        kube_client.expect_update_status().times(0);
        kube_client.expect_add_finalizer().times(0);

        let cats_api = Arc::new(mock_cats_api) as Arc<dyn CatsApi>;
        let kube_client = Arc::new(kube_client) as Arc<dyn KubeApi<Cat>>;

        let result = reconcile(
            Arc::clone(&cat),
            Arc::new(ContextData {
                kube_client: kube_client.clone(),
                cats_client: cats_api.clone(),
            }),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reconcile_executing_update_if_observation_generation_and_meta_observation_not_equal(
    ) {
        let mut kube_client = MockKubeApiClient::new();
        let mut mock_cats_api = MockCatsApi::new();
        let uuid = Uuid::new_v4();

        let cat = Arc::new(Cat {
            metadata: kube::api::ObjectMeta {
                name: Some("whiskers".to_string()),
                generation: Some(1),
                ..Default::default()
            },
            spec: CatSpec {
                name: "Whiskers".to_string(),
                breed: "Siamese".to_string(),
                age: 3,
            },
            status: Some(CatStatus {
                uuid: Some(uuid.to_string()),
                observed_generation: Some(0),
                conditions: vec![],
            }),
        });

        let remote_cat = CatDto {
            uuid: Some(uuid),
            name: cat.spec.name.clone(),
            breed: cat.spec.breed.clone(),
            age: cat.spec.age,
        };

        let cat_clone_1 = Arc::clone(&cat);
        let cat_clone_2 = Arc::clone(&cat);

        mock_cats_api
            .expect_get_cat_by_id()
            .times(1)
            .returning(move |_| Ok(remote_cat.clone()));
        mock_cats_api.expect_create_cat().times(0);

        mock_cats_api
            .expect_update_cat_by_id()
            .withf(move |id, dto| {
                id == uuid.to_string()
                    && dto.name == cat_clone_1.spec.name
                    && dto.breed == cat_clone_1.spec.breed
                    && dto.age == cat_clone_1.spec.age
            })
            .times(1)
            .returning(move |_, dto| Ok(dto));

        kube_client
            .expect_replace()
            .times(1)
            .returning(move |_, _, _| Ok(cat_clone_2.as_ref().clone()));
        kube_client
            .expect_update_status()
            .times(1)
            .returning(|_| Ok(()));
        kube_client
            .expect_create_condition()
            .times(1)
            .withf(move |status, type_, reason, message, observed_generation| {
                status == "Updated"
                    && type_ == "AvailableUpdated"
                    && reason == "Updated the resource"
                    && message == "Resource has been updated"
                    && *observed_generation == Some(1i64)
            })
            .returning(
                |status, type_, reason, message, observed_generation| Condition {
                    status: status.to_string(),
                    type_: type_.to_string(),
                    reason: reason.to_string(),
                    message: message.to_string(),
                    observed_generation: observed_generation,
                    last_transition_time: Time(chrono::Utc::now()),
                },
            );

        let cats_api = Arc::new(mock_cats_api) as Arc<dyn CatsApi>;
        let kube_client = Arc::new(kube_client) as Arc<dyn KubeApi<Cat>>;

        let result = reconcile(
            Arc::clone(&cat),
            Arc::new(ContextData {
                kube_client: kube_client.clone(),
                cats_client: cats_api.clone(),
            }),
        )
        .await;

        assert!(result.is_ok());
    }
}
