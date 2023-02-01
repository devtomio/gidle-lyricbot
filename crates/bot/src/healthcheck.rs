use salvo::prelude::StatusCode;
use salvo::{handler, Response};

#[handler]
pub async fn healthcheck(res: &mut Response) {
    res.set_status_code(StatusCode::OK);
}
