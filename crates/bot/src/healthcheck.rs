use salvo::prelude::StatusCode;
use salvo::{handler, Response};

#[handler]
#[instrument]
pub async fn healthcheck(res: &mut Response) {
    info!("recieved healthcheck request.");

    res.set_status_code(StatusCode::OK);
}
