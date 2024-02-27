use std::fmt::Debug;

use candid::Nat;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

pub async fn post_json<T, R>(url: &str, body: T) -> Result<R, crate::Error>
where
    T: Serialize + Debug,
    R: DeserializeOwned + Debug,
{
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(_e) => return Err(crate::Error::ParseError),
    };
    let host = match parsed_url.host_str() {
        None => return Err(crate::Error::CantParseHost),
        Some(host) => host,
    };

    let request_headers = vec![
        HttpHeader {
            name: "Host".to_string(),
            value: format!("{host}:443"),
        },
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "rag_canister".to_string(),
        },
        HttpHeader {
            name: "Idempotency-Key".to_string(),
            value: "UUID-123456789".to_string(),
        },
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
    ];

    let json_string = match serde_json::to_string(&body) {
        Ok(string) => string,
        Err(_err) => return Err(crate::Error::BodyNonSerializable),
    };
    ic_cdk::api::print(format!("{:?}", json_string));
    let json_utf8: Vec<u8> = json_string.into_bytes();
    let request_body: Option<Vec<u8>> = Some(json_utf8);

    #[derive(Serialize, Deserialize)]
    struct Context {
        does_nothing: u64,
    }
    // legacy code... does nothing!
    let context = Context { does_nothing: 0 };

    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::POST,
        body: request_body,
        // max_response_bytes: None,
        max_response_bytes: Some(5000_u64), //optional for request
        transform: Some(TransformContext::from_name(
            "transform".to_string(),
            serde_json::to_vec(&context).unwrap(),
        )),
        headers: request_headers,
    };
    // let cycles = 300_000_000_000;
    // trail and error
    let cycles = 1_903_155_600;

    match http_request(request, cycles).await {
        Ok((response,)) => {
            let body: Result<R, serde_json::Error> = serde_json::from_slice(&response.body);
            match body {
                Ok(body) => {
                    ic_cdk::api::print(format!("In body OK,{:?}", body));
                    Ok(body)
                }
                Err(e) => {
                    ic_cdk::api::print(format!("Received an error from api: err = {:?}", e));
                    Err(crate::Error::BodyNonSerializable)
                }
            }
        }
        Err(err) => Err(crate::Error::HttpError(err.1)),
    }
}

// Strips all data that is not needed from the original response.
pub fn transform_impl(raw: TransformArgs) -> HttpResponse {
    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
        ..Default::default()
    };

    if res.status == Nat::from(200_u32) {
        res.body = raw.response.body;
    } else {
        ic_cdk::api::print(format!("Received an error from api: err = {:?}", raw));
    }
    return res;
}
