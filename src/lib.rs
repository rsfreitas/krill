extern crate mongodb;
pub use mongodb::bson::{doc, Document};

pub mod database;
pub mod error;
pub mod grpc;
pub mod service;
pub mod extensions;

mod config;
mod definition;
