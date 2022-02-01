use std::sync::Arc;

use mongodb::{
    bson::{doc, Document},
    error::Result as MongoResult,
    options::{ClientOptions, UpdateModifications},
    Client, Cursor,
};

use crate::config::{Config, GetEnv};
use crate::error::Result;
use crate::grpc::rpc;

pub type DatabaseResult<T> = std::result::Result<T, tonic::Status>;

#[derive(Debug)]
pub struct Database {
    client: Client,
    info: Info,
}

#[derive(Clone, Debug)]
pub struct Credentials {
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub tls_cacert_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Info {
    pub database_name: Option<String>,
    pub collection: Option<String>,
}

impl Default for Credentials {
    fn default() -> Self {
        Credentials {
            host: Some("localhost".to_string()),
            port: Some(27017),
            username: None,
            password: None,
            tls_cacert_path: None,
        }
    }
}

impl Default for Info {
    fn default() -> Self {
        Info {
            database_name: Config::get_os_env("DATABASE_NAME", None),
            collection: Config::get_os_env("DATABASE_COLLECTION_NAME", None),
        }
    }
}

impl Database {
    pub(crate) async fn new(credentials: &Credentials, info: &Info) -> Result<Arc<Self>> {
        let uri = Database::get_database_uri(&Database::credentials(credentials))?;
        let client_options = ClientOptions::parse(&uri).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        Ok(Arc::new(Database {
            client,
            info: info.clone(),
        }))
    }

    fn get_database_uri(credentials: &Credentials) -> Result<String> {
        // We allow using empty username and password to help local testing.
        if credentials.username.is_none() && credentials.password.is_none() {
            return Ok(format!(
                "mongodb://{}:{}",
                credentials.host.as_ref().unwrap(),
                credentials.port.unwrap()
            ));
        }

        // TODO: validate credentials?
        Ok(format!(
            "mongodb://{}:{}@{}:{}/tls=true?replicaSet=rs0&readPreference=secondaryPreferred&retryWrites=false",
            credentials.username.as_ref().unwrap(),
            credentials.password.as_ref().unwrap(),
            credentials.host.as_ref().unwrap(),
            credentials.port.unwrap(),
        ))
    }

    fn credentials(default_credentials: &Credentials) -> Credentials {
        Credentials {
            host: Config::get_os_env("DATABASE_HOST", default_credentials.host.clone()),
            port: Config::get_os_env("DATABASE_PORT", default_credentials.port),
            username: Config::get_os_env("DATABASE_USERNAME", default_credentials.username.clone()),
            password: Config::get_os_env("DATABASE_PASSWORD", default_credentials.password.clone()),
            tls_cacert_path: Config::get_os_env(
                "DATABASE_TLS_CACERT_PATH",
                default_credentials.tls_cacert_path.clone(),
            ),
        }
    }

    /// Inserts a new record into the current selected collection.
    pub async fn insert<T: serde::Serialize + prost::Message>(
        &self,
        source: &T,
    ) -> DatabaseResult<()> {
        let db = self
            .client
            .database(self.info.database_name.as_ref().unwrap());

        let collection = db.collection::<T>(self.info.collection.as_ref().unwrap());

        match collection.insert_one(source, None).await {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(rpc::Error::new(rpc::ErrorCode::Internal, Some(&e.to_string())).to_status())
            }
        }
    }

    /// Finds a single record from the current collection using a custom filter.
    pub async fn find_one<T: prost::Message + serde::de::DeserializeOwned + Unpin>(
        &self,
        filter: Document,
    ) -> DatabaseResult<T> {
        let db = self
            .client
            .database(self.info.database_name.as_ref().unwrap());

        let collection = db.collection::<T>(self.info.collection.as_ref().unwrap());
        match collection.find_one(filter, None).await {
            Ok(record) => match record {
                Some(data) => Ok(data),
                None => Err(rpc::Error::new(rpc::ErrorCode::NotFound, None).to_status()),
            },
            Err(e) => {
                Err(rpc::Error::new(rpc::ErrorCode::Internal, Some(&e.to_string())).to_status())
            }
        }
    }

    /// Finds a single record from the current collection by using an ID as filter.
    pub async fn find_one_by_id<T: prost::Message + serde::de::DeserializeOwned + Unpin>(
        &self,
        id: &str,
    ) -> DatabaseResult<T> {
        let db = self
            .client
            .database(self.info.database_name.as_ref().unwrap());

        let collection = db.collection::<T>(self.info.collection.as_ref().unwrap());
        let filter = doc! {"_id": id};

        match collection.find_one(filter, None).await {
            Ok(record) => match record {
                Some(data) => Ok(data),
                None => Err(rpc::Error::new(rpc::ErrorCode::NotFound, None).to_status()),
            },
            Err(e) => {
                Err(rpc::Error::new(rpc::ErrorCode::Internal, Some(&e.to_string())).to_status())
            }
        }
    }

    /// Find one or more records from the current collection using a custom filter.
    pub async fn find_many<T: prost::Message + serde::de::DeserializeOwned + Unpin>(
        &self,
        filter: Document,
    ) -> MongoResult<Cursor<T>> {
        let db = self
            .client
            .database(self.info.database_name.as_ref().unwrap());

        let collection = db.collection::<T>(self.info.collection.as_ref().unwrap());
        collection.find(filter, None).await
    }

    /// Updates a single record into the current collection.
    pub async fn update<T: serde::Serialize + serde::de::DeserializeOwned + prost::Message>(
        &self,
        id: &str,
        source: Document,
    ) -> DatabaseResult<T> {
        let db = self
            .client
            .database(self.info.database_name.as_ref().unwrap());

        let collection = db.collection::<T>(self.info.collection.as_ref().unwrap());
        let filter = doc! {"_id": id};
        let up = doc! {"$set": source};

        let result = collection
            .find_one_and_update(filter, UpdateModifications::Document(up), None)
            .await;

        match result {
            Ok(record) => match record {
                Some(data) => Ok(data),
                None => Err(rpc::Error::new(rpc::ErrorCode::NotFound, None).to_status()),
            },
            Err(e) => {
                Err(rpc::Error::new(rpc::ErrorCode::Internal, Some(&e.to_string())).to_status())
            }
        }
    }

    /// Deletes a single record from the current selected collection.
    pub async fn delete<T: serde::Serialize + serde::de::DeserializeOwned + prost::Message>(
        &self,
        id: &str,
    ) -> DatabaseResult<T> {
        let db = self
            .client
            .database(self.info.database_name.as_ref().unwrap());

        let collection = db.collection::<T>(self.info.collection.as_ref().unwrap());
        let filter = doc! {"_id": id};
        let result = collection.find_one_and_delete(filter, None).await;

        match result {
            Ok(record) => match record {
                Some(data) => Ok(data),
                None => Err(rpc::Error::new(rpc::ErrorCode::NotFound, None).to_status()),
            },
            Err(e) => {
                Err(rpc::Error::new(rpc::ErrorCode::Internal, Some(&e.to_string())).to_status())
            }
        }
    }
}
