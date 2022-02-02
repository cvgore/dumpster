use rocket::http::Status;
use rocket::local::asynchronous::Client;

#[rocket::async_test]
async fn index_loads() {
    let client = Client::untracked(crate::rocket()).await.expect("valid rocket instance");

    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert!(response.into_string().await.expect("body present").contains("dumpster"));
}