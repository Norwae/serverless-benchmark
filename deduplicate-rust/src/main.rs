use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::convert::identity;
use std::io::{copy, Read, Write};
use std::ptr::hash;
use std::str::FromStr;

use bytes::{Bytes, BytesMut};
use futures::{future::join_all, FutureExt, join};
use lambda_runtime::{Context, Error, handler_fn};
use lazy_static::lazy_static;
use ring::digest::{self, Digest, SHA512};
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, GetItemInput, GetItemOutput, PutItemInput};
use rusoto_s3::{DeleteObjectRequest, GetObjectRequest, S3, S3Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio;
use tokio::io::AsyncReadExt;

mod dynamo_serde_mapping;

lazy_static! {
    static ref DYNAMO_CLIENT: DynamoDbClient = DynamoDbClient::new(region());
    static ref S3_CLIENT: S3Client = S3Client::new(region());
}

fn region() -> Region {
    let region = std::env::var("AWS_REGION").expect("AWS_REGION provided by runtime");
    Region::from_str(&region).expect("Region sensible")
}

#[derive(Debug, Deserialize, Serialize)]
struct FileAlias {
    file_name: String,
    canonical_name: String
}

#[derive(Debug, Deserialize, Serialize)]
struct CanonicalName {
    hash_base_64: String,
    canonical_name: String
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(handle);
    lambda_runtime::run(func).await
}

struct ObjectHash {
    bucket: String,
    key: String,
    hash: String
}

async fn handle(envelope: Value, _: Context) -> Result<&'static str, Error> {
    let records = envelope["Records"].as_array().expect("Records array at root");

    let dedup = records.into_iter().map(|record| async move {
        let hashed = hash_s3_object(&record).await;
        deduplicate( hashed).await
    }).collect::<Vec<_>>();

    join_all(dedup).await;

    Ok("done")
}

async fn delete_duplicate(hash: &ObjectHash) {
    let s3: &S3Client = &*S3_CLIENT;
    s3.delete_object(DeleteObjectRequest {
        bucket: hash.bucket.clone(),
        key: hash.key.clone(),
        ..Default::default()
    }).await.expect("Delete succeeded");
}

async fn put_canonical(hash: &ObjectHash) {
    let dynamo: &DynamoDbClient = &*DYNAMO_CLIENT;
    dynamo.put_item(PutItemInput {
        table_name: "canonicalsRust".to_string(),
        item: dynamo_serde_mapping::serialize_to_dynamo(CanonicalName {
            hash_base_64: hash.hash.clone(),
            canonical_name: hash.key.clone()
        }).expect("Ser canonical succeeds"),
        ..Default::default()
    }).await.expect("Put canonical succeeded");
    println!("Canonical put successfully")
}

async fn put_file_alias(canonical_name: String, file_name: String) {
    let dynamo = &*DYNAMO_CLIENT;
    let alias = FileAlias {
        canonical_name,
        file_name
    };
    println!("Putting file alias {:?}", alias);
    dynamo.put_item(PutItemInput {
        table_name: "fileAliasRust".to_string(),
        item: dynamo_serde_mapping::serialize_to_dynamo(alias).expect("ser filename ok"),
        ..Default::default()
    }).await.expect("save file ok");

    println!("Alias put successfully");
}

async fn deduplicate(hash: ObjectHash) {

    let found = get_previous_canonical(&hash).await;

    let (canonical, housekeeping) = match found {
        None => {
            println!("No previous canonical found, putting this");
            let name = hash.key.clone();
            (name, put_canonical(&hash).boxed())
        }
        Some(record) => {
            println!("Previous canonical found ({}), deleting duplicate", &record.canonical_name);
            (record.canonical_name, delete_duplicate(&hash).boxed())
        }
    };

    let put = put_file_alias(canonical, hash.key.clone());
    join!(housekeeping, put);
}

async fn get_previous_canonical(hash: &ObjectHash) -> Option<CanonicalName> {
    let dynamo = &*DYNAMO_CLIENT;
    let output = dynamo.get_item(GetItemInput {
        key: dynamo_serde_mapping::serialize_to_dynamo(json!({"hash_base_64": hash.hash})).unwrap(),
        table_name: "canonicalsRust".to_string(),
        ..Default::default()
    }).await.expect("dynamodb available");
    output.item.map(|item|dynamo_serde_mapping::deserialize_from_dynamo(item).expect("Deser canonical ok"))
}

async fn hash_s3_object(record: &Value) -> ObjectHash {
    let s3: &S3Client = &*S3_CLIENT;
    let bucket = record["s3"]["bucket"]["name"].as_str().expect("Bucket name must be string").to_string();
    let key = record["s3"]["object"]["key"].as_str().expect("Object key must be string").to_string();

    let download_headers = s3.get_object(GetObjectRequest {
        bucket: bucket.clone(),
        key: key.clone(),
        ..Default::default()
    }).await.expect("Headers expected to be okay");

    let download_body = download_headers.body.expect("Object expected to be present");
    let mut hasher = digest::Context::new(&SHA512);

    let mut buffer = BytesMut::with_capacity(1 << 16);
    let mut reader = download_body.into_async_read();
    loop {
        let received = reader.read(&mut buffer).await.expect("Body read okay");
        if received == 0 {
            break
        }

        hasher.update(&buffer[0..received]);
    }

    let hash = base64::encode(hasher.finish());
    println!("Fully hashed {}", &key);
    ObjectHash { bucket, key, hash }
}