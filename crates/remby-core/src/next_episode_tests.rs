use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn mock_server(responses: Vec<serde_json::Value>) -> (EmbyClient, tokio::task::JoinHandle<()>) {
    init_device_id();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        for (index, body) in responses.into_iter().enumerate() {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            loop {
                let mut buffer = [0; 2048];
                let count = stream.read(&mut buffer).await.unwrap();
                assert!(count > 0);
                request.extend_from_slice(&buffer[..count]);
                if request.windows(4).any(|part| part == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8(request).unwrap();
            let path = request.split_whitespace().nth(1).unwrap();
            if index > 0 {
                let url = Url::parse(&format!("http://localhost{path}")).unwrap();
                let query: std::collections::HashMap<_, _> = url.query_pairs().collect();
                assert_eq!(url.path(), "/Shows/series/Episodes");
                assert_eq!(query["SortBy"], "ParentIndexNumber,IndexNumber");
                assert_eq!(query["SortOrder"], "Ascending");
                assert_eq!(query["StartIndex"], (index - 1).to_string());
            }
            let body = body.to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body,
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        }
    });
    let client = EmbyClient {
        base_url: format!("http://{address}"),
        token: "test".into(),
        user_id: "test".into(),
        http: Client::builder().no_proxy().timeout(std::time::Duration::from_secs(3)).build().unwrap(),
    };
    (client, server)
}

#[tokio::test]
async fn finds_next_episode_across_pages_and_seasons() {
    let current = json!({"Id": "current", "Type": "Episode", "SeriesId": "series", "ParentIndexNumber": 1, "IndexNumber": 10});
    let (client, server) = mock_server(vec![
        current.clone(),
        json!({"Items": [current], "TotalRecordCount": 2}),
        json!({"Items": [{"Id": "next", "Type": "Episode", "ParentIndexNumber": 2, "IndexNumber": 1}], "TotalRecordCount": 2}),
    ]).await;
    let next = client.get_next_episode("current").await.unwrap().unwrap();
    assert_eq!(next.id, "next");
    assert_eq!(next.parent_index_number, Some(2));
    server.await.unwrap();
}

#[tokio::test]
async fn final_episode_has_no_next_episode() {
    let current = json!({"Id": "last", "Type": "Episode", "SeriesId": "series"});
    let (client, server) = mock_server(vec![
        current.clone(),
        json!({"Items": [current], "TotalRecordCount": 1}),
    ]).await;
    assert!(client.get_next_episode("last").await.unwrap().is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn movies_do_not_request_episodes() {
    let (client, server) = mock_server(vec![json!({"Id": "movie", "Type": "Movie"})]).await;
    assert!(client.get_next_episode("movie").await.unwrap().is_none());
    server.await.unwrap();
}
