use axum::Router;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, openapi::OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

pub struct ApiKeySecuritySchemeAddon;

impl Modify for ApiKeySecuritySchemeAddon {
    fn modify(&self, openapi: &mut OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
            )
        }
    }
}

pub struct JwtSecuritySchemeAddon;

impl Modify for JwtSecuritySchemeAddon {
    fn modify(&self, openapi: &mut OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

pub fn router_with_docs(router: OpenApiRouter) -> Router {
    let openapi_json = "/http/v1/openapi.json";
    let (router, api) = router.split_for_parts();
    let router = router
        .merge(SwaggerUi::new("/swagger-ui").url(openapi_json, api.clone()))
        .merge(Redoc::with_url("/redoc", api.clone()))
        .merge(RapiDoc::new(openapi_json).path("/rapidoc"))
        .merge(Scalar::with_url("/", api.clone()));
    router
}

#[macro_export]
macro_rules! route_docs_with_name {
    (
        $fn_expr:ident,
        desc=$desc:expr,
        tag=$tag:expr,
        res = $( ($status:literal, $resp_ty:ty) ),+ $(,)?
    ) => {
        pub fn $fn_expr(op: TransformOperation) -> TransformOperation {
            let op = op.description($desc).tag($tag);
            $( let op = op.response::<$status, $resp_ty>(); )+
            op
        }

    };
}

#[macro_export]
macro_rules! route_docs {
    (
        desc=$desc:expr,
        tag=$tag:expr,
        res = $( ($status:literal, $resp_ty:ty) ),+ $(,)?
    ) => {
        |op: TransformOperation| -> TransformOperation {
            let op = op.description($desc).tag($tag);
            $( let op = op.response::<$status, $resp_ty>(); )+
            op
        }
    };
}

#[macro_export]
macro_rules! endpoint {
(
    $(#[$attr:meta])*
    $process_fn:ident($($param:ident: $param_ty:ty),*)
) => {
    paste! {
        $(#[$attr])*
        async fn [<$process_fn _endpoint>](
            $($param: $param_ty),*
        ) -> impl IntoResponse {
            $process_fn($($param),*).await
        }
    }
    };
}

#[macro_export]
macro_rules! register_routes {
    ($($fn_name:ident),+ $(,)?) => {
        {
            use utoipa_axum::routes;
            let mut router = OpenApiRouter::new();
            $(
                router = router.routes(routes!($fn_name));
            )+
            router
        }
    };
}
