use utoipa::OpenApi;
use crate::routes::notification::{NotificationInput, NotificationItem};

#[derive(OpenApi)]
#[openapi(
    info(title = "OneSociety API", version = "1.0"),
    components(
        schemas(NotificationInput, NotificationItem)
    ),
    paths()
)]
pub struct ApiDoc;

