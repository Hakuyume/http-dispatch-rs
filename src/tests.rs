use super::{Client, Error, Json, TypedHeader};
use headers::UserAgent;
use http::{Method, StatusCode, Uri};
use serde::Deserialize;

fn client() -> Client {
    Client::hyper()
}

// https://docs.github.com/en/free-pro-team@latest/rest/meta/meta?apiVersion=2022-11-28#get-github-meta-information
#[tokio::test]
async fn test_github_meta() {
    #[derive(Deserialize)]
    struct Response {
        ssh_keys: Vec<String>,
    }

    let response = client()
        .send::<(_, Uri, _, _), Json<Response>>((
            Method::GET,
            "https://api.github.com/meta".parse().unwrap(),
            TypedHeader(UserAgent::from_static(env!("CARGO_CRATE_NAME"))),
            (),
        ))
        .await
        .unwrap();
    assert!(!response.0.ssh_keys.is_empty());
}

// https://docs.github.com/en/free-pro-team@latest/rest/meta/meta?apiVersion=2022-11-28#get-github-meta-information
#[tokio::test]
async fn test_github_meta_without_user_agent() {
    let e = client()
        .send::<(_, Uri, _), ()>((
            Method::GET,
            "https://api.github.com/meta".parse().unwrap(),
            (),
        ))
        .await
        .unwrap_err();
    assert!(matches!(
        e,
        Error::Http {
            status: StatusCode::FORBIDDEN,
            ..
        },
    ));
}
