use serde::{Deserialize, Serialize};
use spin_sdk::http::{Headers, IncomingRequest, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;
use futures::SinkExt;

/// A simple Spin HTTP component.
#[http_component]
async fn handle_{{project-name | snake_case}}(req: IncomingRequest, resp_out: ResponseOutparam) {
    let req_body = req.into_body().await.unwrap();

    let cmd: SlackCommand = match serde_qs::from_bytes(&req_body) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Deserialisation error {e:?}");
            bad_request(resp_out).await;
            return;
        }
    };

    let headers = Headers::new();
    headers.set(&"content-type".to_string(), &["text/plain".as_bytes().to_vec()]).unwrap();
    let resp = OutgoingResponse::new(headers);
    resp.set_status_code(200).unwrap();
    let mut ogbod = resp.take_body();

    resp_out.set(resp);

    // Send the acknowledgement response. If you can do all your processing within Slack's
    // 3-second timeout, you can send your full response here.
    ogbod
        .send("Processing... processing...".as_bytes().to_vec())
        .await
        .unwrap();

    // If you can do all your processing as part of the acknowledgement response, you can delete
    // the rest of this function.

    // End the request so Slack doesn't time out while you do long-running work.
    ogbod.flush().await.unwrap();
    ogbod.close().await.unwrap();
    drop(ogbod);

    // Pretend to do long-running work. *** REMOVE THIS FROM YOUR ACTUAL COMMAND! ***
    std::thread::sleep(std::time::Duration::from_secs(6));

    // This template creates an ephemeral message response.  See the kinds of response
    // you can send at https://api.slack.com/interactivity/handling#responses
    let response_message = SlackMessageResponse {
        text: format!("Here is the result of your '{}' message", cmd.text),
        response_type: None, // set to `Some("in_channel".to_string())` to respond to the channel,
    };

    let response_callback = spin_sdk::http::Request::builder()
        .uri(&cmd.response_url)
        .method(spin_sdk::http::Method::Post)
        .header("content-type", "application/json")
        .body(serde_json::to_vec_pretty(&response_message).unwrap())
        .build();

    // Send the message response. In this template we ignore any response-to-the-response from
    // Slack but you can extend this as needed.
    let _unused: spin_sdk::http::Response = spin_sdk::http::send(response_callback).await.unwrap();
}

// See https://api.slack.com/interactivity/slash-commands#app_command_handling for other fields you can capture
#[derive(Deserialize)]
struct SlackCommand {
    text: String,
    response_url: String,
}

// See https://api.slack.com/interactivity/handling#message_responses for additional response fields
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct SlackMessageResponse {
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response_type: Option<String>,
}

async fn bad_request(resp_out: ResponseOutparam) {
    let headers = Headers::new();
    headers.set(&"content-type".to_string(), &["text/plain".as_bytes().to_vec()]).unwrap();
    let resp = OutgoingResponse::new(headers);
    resp.set_status_code(200).unwrap();
    let mut ogbod = resp.take_body();

    resp_out.set(resp);

    ogbod
        .send("Incmplete Slack request\n".as_bytes().to_vec())
        .await
        .unwrap();
}
