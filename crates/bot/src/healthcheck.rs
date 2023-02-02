use salvo::http::header::CONTENT_TYPE;
use salvo::http::{HeaderMap, HeaderValue, StatusCode};
use salvo::{handler, Response};

#[handler]
#[instrument(skip(res))]
pub async fn healthcheck(res: &mut Response) {
    info!("recieved healthcheck request.");

    let mut headers = HeaderMap::new();
    headers.append(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

    res.set_status_code(StatusCode::OK);
    res.set_headers(headers);
    res.render("ok.");
}
