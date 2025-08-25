use utoipa::OpenApi;
use crate::routes::notification::{NotificationInput, NotificationItem};

#[derive(OpenApi)]
#[openapi(
    info(title = "OneSociety API", version = "1.0"),
    components(
        schemas(NotificationInput, NotificationItem)
    ),
    paths(
        crate::routes::notification::list_user_notifications,
        crate::routes::notification::create_notification,
        crate::routes::notification::mark_read
    )
)]
pub struct ApiDoc;

