# micro

micro is a rust framework for building microservices.

## Overview

micro is a framework for building microservices in rust using protobuf as
their API definition.

## Features

* TOML settings file
* Structured logging messages
* Environment variables support
* gRPC microservices

## Getting Started

To make use of micro, one can do the following:
```rust
extern crate micro;
extern crate tonic;

pub mod examplev1 {
    tonic::include_proto!("example.v1");
}

use examplev1::grpc_service_server::{GrpcService, GrpcServiceServer};
use micro::grpc::rpc;
use micro::service::{
    builder::ServiceBuilder,
    Service,
};

#[derive(Default)]
struct Server {}

#[tonic::async_trait]
impl GrpcService for Server {
    // Implement the GrpcService trait
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = ServiceBuilder::default().build();
    Service::serve_as_grpc(&service.unwrap(), GrpcServiceServer::new(Server::default())).await
}
```

## TODO

* HTTP microservices
* Pubsub microservices
* Task microservices (cronjob)
* Database access
* Pluggable interfaces

## License

Apache 2.0

## Acknowledgments

* [go-micro](https://github.com/asim/go-micro)

