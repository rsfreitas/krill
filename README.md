# krill

krill is a rust framework for building microservices.

## Overview

krill is a framework for building microservices in rust using protobuf as
their API definition.

## Features

* TOML settings file
* Structured logging messages
* Environment variables support
* gRPC microservices

## Getting Started

To make use of krill, one can do the following:
```rust
extern crate krill;

use krill::service::{
    builder::ServiceBuilder,
    Service,
};

#[derive(Default)]
struct Server {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = ServiceBuilder::default().build();
    Service::serve_as_grpc(&service.unwrap(), GrpcServiceServer::new(Server::default())).await
}
```

### Creating a gRPC microservice

Service protobuf API definition:
```protobuf
syntax = "proto3";

package example;

service ExampleService {
  rpc GetExample(GetExampleRequest) returns (GetExampleResponse);
  rpc CreateExample(CreateExampleRequest) returns (CreateExampleResponse);
  rpc UpdateExample(UpdateExampleRequest) returns (UpdateExampleResponse);
  rpc DeleteExample(DeleteExampleRequest) returns (DeleteExampleResponse);
}

message Example {
  string id = 1;
  string name = 2;
  int32 value = 3;
}

message GetExampleRequest {
  string id = 1;
}

message GetExampleResponse {
  Example example = 1;
}

message CreateExampleRequest {
  string name = 1;
  int32 value = 2;
}

message CreateExampleResponse {
  Example example = 1;
}

message UpdateExampleRequest {
  string id = 1;
  string name = 2;
  int32 value = 3;
}

message UpdateExampleResponse {
  Example example = 1;
}

message DeleteExampleRequest {
  string id = 1;
}

message DeleteExampleResponse {
  Example example = 1;
}
```

Service krill source:
```rust
extern crate krill;
extern crate tonic;

pub mod example {
    tonic::include_proto!("example");
}

use example::example_service_server::{ExampleService, ExampleServiceServer};
use krill::grpc::rpc;
use krill::{
    extensions::database::Id,
    service::{builder::ServiceBuilder, Service},
};

#[derive(Default)]
struct Server {}

#[tonic::async_trait]
impl ExampleService for Server {
    async fn get_example(
        &self,
        request: rpc::Request<example::GetExampleRequest>,
    ) -> rpc::Response<example::GetExampleResponse> {
        let service = Service::from_request(request);
        let example::GetExampleRequest { id } = &request.into_inner();
        let ex = service
            .database()
            .find_one_by_id::<example::Example>(id)
            .await?;

        rpc::ok(example::GetExampleResponse { example: Some(ex) })
    }

    async fn create_example(
        &self,
        request: rpc::Request<example::CreateExampleRequest>,
    ) -> rpc::Response<example::CreateExampleResponse> {
        let service = Service::from_request(request);
        let example::CreateExampleRequest { name, value } = &request.into_inner();
        let res = example::Example {
            id: Id::new("ex"),
            name: name.to_string(),
            value: *value,
        };

        let _ service.database().insert(&res).await?;
        rpc::ok(example::CreateExampleResponse { example: Some(res) })
    }

    async fn update_example(
        &self,
        request: rpc::Request<example::UpdateExampleRequest>,
    ) -> rpc::Response<example::UpdateExampleResponse> {
        let service = Service::from_request(request);
        let example::UpdateExampleRequest { id, name, value } = &request.into_inner();
        let up = service
            .database()
            .update::<example::Example>(id, krill::doc! {"name": name, "value": value})
            .await?;

        rpc::ok(example::UpdateExampleResponse { example: Some(up) })
    }

    async fn delete_example(
        &self,
        request: rpc::Request<example::DeleteExampleRequest>,
    ) -> rpc::Response<example::DeleteExampleResponse> {
        let service = Service::from_request(request);
        let example::DeleteExampleRequest { id } = &request.into_inner();
        let d = service.database().delete::<example::Example>(id).await?;
        rpc::ok(example::DeleteExampleResponse { example: Some(d) })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = ServiceBuilder::default().build();
    Service::serve_as_grpc(&service.unwrap(), ExampleServiceServer::new(Server::default())).await
}
```

## TODO

* Pubsub microservices
* Task microservices (cronjob)
* Pluggable interfaces

## License

Apache 2.0

## Acknowledgments

* [go-micro](https://github.com/asim/go-micro)

