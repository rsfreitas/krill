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

use micro::service::{
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

Service micro source:
```rust
extern crate micro;
extern crate tonic;

pub mod example {
    tonic::include_proto!("example");
}

use example::example_service_server::{ExampleService, ExampleServiceServer};
use micro::grpc::rpc;
use micro::{
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
            .await;

        match ex {
            Ok(example) => match example {
                Some(e) => rpc::ok(example::GetExampleResponse { example: Some(e) }),
                None => rpc::error(rpc::ErrorCode::NotFound),
            },
            Err(_) => rpc::error(rpc::ErrorCode::Internal),
        }
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

        match service.database().insert(&res).await {
            Ok(_) => rpc::ok(example::CreateExampleResponse { example: Some(res) }),
            Err(_) => rpc::error(rpc::ErrorCode::Internal),
        }
    }

    async fn update_example(
        &self,
        request: rpc::Request<example::UpdateExampleRequest>,
    ) -> rpc::Response<example::UpdateExampleResponse> {
        let service = Service::from_request(request);
        let example::UpdateExampleRequest { id, name, value } = &request.into_inner();
        let up = service
            .database()
            .update::<example::Example>(id, micro::doc! {"name": name, "value": value})
            .await;

        match up {
            Ok(example) => match example {
                Some(e) => rpc::ok(example::UpdateExampleResponse { example: Some(e) }),
                None => rpc::error(rpc::ErrorCode::NotFound),
            },
            Err(_) => rpc::error(rpc::ErrorCode::Internal),
        }
    }

    async fn delete_example(
        &self,
        request: rpc::Request<example::DeleteExampleRequest>,
    ) -> rpc::Response<example::DeleteExampleResponse> {
        let service = Service::from_request(request);
        let example::DeleteExampleRequest { id } = &request.into_inner();
        let d = service.database().delete::<example::Example>(id).await;

        match d {
            Ok(example) => match example {
                Some(e) => rpc::ok(example::DeleteExampleResponse { example: Some(e) }),
                None => rpc::error(rpc::ErrorCode::NotFound),
            },
            Err(_) => rpc::error(rpc::ErrorCode::Internal),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = ServiceBuilder::default().build();
    Service::serve_as_grpc(&service.unwrap(), ExampleServiceServer::new(Server::default())).await
}
```

## TODO

* HTTP microservices
* Pubsub microservices
* Task microservices (cronjob)
* Pluggable interfaces

## License

Apache 2.0

## Acknowledgments

* [go-micro](https://github.com/asim/go-micro)

