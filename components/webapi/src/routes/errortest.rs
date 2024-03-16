use crate::{errors::ApiErrorResponse, ApiResult};

use tracing::instrument;

#[instrument]
pub async fn handle_errortest() -> ApiResult<()> {
    Err(ApiErrorResponse::InternalError)
}
