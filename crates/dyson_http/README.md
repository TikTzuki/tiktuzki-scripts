# dyson_http

Dyson HTTP helpers for Axum services. This crate gives you:

- A ready-to-merge health check route: `/actuator/health`.
- A simple way to serve interactive API docs (Swagger UI, Redoc, RapiDoc, Scalar) backed by `utoipa`.
- Small macros to make wiring routes and documenting endpoints ergonomic.

Works great for small services or bootstrapping new ones quickly.

## Install

Add these to your `Cargo.toml`:

```toml
[dependencies]
axum = "0.8"
dyson_http = "0"
# You write the OpenAPI derives and use the axum routes macro in your app,
# so you also need these dependencies in your crate:
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-axum = "0"
```

If you're working inside a workspace with this repo checked out, you can also use a path dependency while developing:

```toml
[dependencies]
dyson_http = { path = "../../crates/dyson_http" }
```

## Features

- Health check route
  - `dyson_http::http::health_check_routes()` merges a simple `GET /actuator/health` => `OK`.
- API documentation router
  - `dyson_http::http::docs::router_with_docs(OpenApiRouter)` exposes:
    - Swagger UI at `/swagger-ui`
    - Redoc at `/redoc`
    - RapiDoc at `/rapidoc`
    - Scalar (Minimal UI) at `/`
    - OpenAPI JSON is served from `/http/v1/openapi.json`.
- Doc helpers
  - Security scheme add-ons: `ApiKeySecuritySchemeAddon`, `JwtSecuritySchemeAddon` (implement `utoipa::Modify`).
  - Route helpers/macros: `register_routes!`, `route_docs!`, and `route_docs_with_name!` to help enrich operations.

## Quick start

### 1) Health check only

```rust
use axum::{Router, http::StatusCode};

#[tokio::main]
async fn main() {
    // Start with any router you have
    let app = Router::new()
        // Merge a simple health check endpoint: GET /actuator/health -> "OK"
        .merge(dyson_http::http::health_check_routes());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 2) Add interactive API docs (Swagger UI, Redoc, RapiDoc, Scalar)

The example below shows a minimal API with a documented endpoint using `utoipa`. The `register_routes!` macro helps collect routes for your OpenAPI and the `router_with_docs` function exposes the documentation UIs.

```rust
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;

#[derive(Serialize, ToSchema)]
struct HelloResponse { message: String }

/// Simple hello endpoint
#[utoipa::path(
    get,
    path = "/hello",
    tag = "Hello",
    responses(
        (status = 200, description = "Hello message", body = HelloResponse)
    )
)]
async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse { message: "Hello, world!".into() })
}

#[derive(OpenApi)]
#[openapi(
    // All your route functions go here
    paths(hello),
    // You can add schemas, tags, and modifiers as needed
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // Collect your routes into an OpenApi-aware router
    let api_router: OpenApiRouter = dyson_http::register_routes!(hello);

    // Turn that into a service router with docs (Swagger UI, Redoc, RapiDoc, Scalar)
    let app: Router = dyson_http::http::docs::router_with_docs(api_router);

    // Add health check (optional)
    let app = app.merge(dyson_http::http::health_check_routes());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

Docs will be available at:
- Swagger UI: http://localhost:3000/swagger-ui
- Redoc: http://localhost:3000/redoc
- RapiDoc: http://localhost:3000/rapidoc
- Scalar: http://localhost:3000/
- OpenAPI JSON: http://localhost:3000/http/v1/openapi.json

### 3) Optional: Add security schemes to your OpenAPI

If you need API key or JWT bearer auth schemes, you can apply our modifiers to your `OpenApi`:

```rust
use utoipa::Modify;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(hello),
    modifiers(&dyson_http::http::docs::ApiKeySecuritySchemeAddon, &dyson_http::http::docs::JwtSecuritySchemeAddon)
)]
struct ApiDoc;
```

## License

Licensed under either of
- Apache License, Version 2.0
- MIT license

at your option.

