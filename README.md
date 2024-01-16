# axum_auto_routes

`axum_auto_routes` is a Rust procedural macro library designed to simplify route handling in Axum web applications. It allows you to annotate async functions with HTTP methods and paths, automatically registering them with Axum's router system. This library streamlines the process of setting up web routes, making your code cleaner and more maintainable.

## Installation

To start using `axum_auto_routes`, add it to your Cargo.toml dependencies:

```toml
[dependencies]
axum_auto_routes = { git = "https://github.com/Th0rgal/axum_auto_routes" }
```

Make sure to also include its dependencies, particularly `axum` and `ctor`:

```toml
[dependencies]
axum = "0.5"  # Adjust the version as necessary
ctor = "0.1"
```

## Usage

First, set up a global route registry in your main application:

```rs
use axum::Router;
use std::sync::Mutex;

lazy_static::lazy_static! {
    pub static ref ROUTE_REGISTRY: Mutex<Vec<Box<dyn WithState>>> = Mutex::new(Vec::new());
}
```

You will need this Trait:
```rs
use axum::{body::Body, Router};
use std::sync::Arc;

pub trait WithState: Send {
    fn to_router(self: Box<Self>, shared_state: Arc<AppState>) -> Router;

    fn box_clone(&self) -> Box<dyn WithState>;
}

impl WithState for Router<Arc<AppState>, Body> {
    fn to_router(self: Box<Self>, shared_state: Arc<AppState>) -> Router {
        self.with_state(shared_state)
    }

    fn box_clone(&self) -> Box<dyn WithState> {
        Box::new((*self).clone())
    }
}

impl Clone for Box<dyn WithState> {
    fn clone(&self) -> Box<dyn WithState> {
        self.box_clone()
    }
}
```

Then, use the `#[route]` macro from `axum_auto_routes` to annotate your async route handler functions. You can specify the HTTP method, the route path, and optionally, the module path if the function is not in the root module.

### Example

`main.rs`:

```rs
mod submodule;
use axum_auto_routes::route;
use axum::{Router, Server};
use std::net::SocketAddr;

// Define a route in the root module
#[route(get, "/")]
async fn root() -> &'static str {
    "Welcome to the root!"
}

#[tokio::main]
async fn main() {
    let app = ROUTE_REGISTRY.lock().unwrap().clone().into_iter().fold(
        Router::new().with_state(shared_state.clone()).layer(cors),
        |acc, r| acc.merge(r.to_router(shared_state.clone())),
    );
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

`submodule.rs`:

```rs
use axum_auto_routes::route;
// Define a route in a submodule (if applicable)
#[route(get, "/example", crate::submodule)]
async fn example_function() {
    // Function implementation...
}
```