use tracing_test::traced_test;

mod utils;

#[tokio::test]
#[traced_test]
async fn test_world_host() {
    let utils::TestServer {
        domain_social,
        domain_world,
        task_social,
        task_world,
    } = utils::setup_test_server().await;

    // DID document is available.
    let response = reqwest::get(format!("http://{}/.well-known/did.json", domain_world))
        .await
        .unwrap();
    let json: serde_json::Value = response.json().await.unwrap();
    let root = json.as_object().unwrap();
    let service = root.get("service").unwrap().as_array().unwrap();
    let service_endpoint = service[0]
        .as_object()
        .unwrap()
        .get("serviceEndpoint")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(service_endpoint, format!("http://{}", domain_social));

    if task_social.is_finished() {
        panic!("Server finished")
    }

    if task_world.is_finished() {
        panic!("Server finished")
    }

    task_social.abort();
    task_world.abort();
}
