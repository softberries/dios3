use crate::model::local_selected_item::LocalSelectedItem;
use crate::model::s3_data_item::{BucketInfo, FileInfo, S3DataItem};
use crate::model::s3_selected_item::S3SelectedItem;
use crate::settings::file_credentials::FileCredential;
use aws_sdk_s3::config::{Credentials, Region};
use dioxus::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{
    fs,
    path::PathBuf,
    pin::Pin
};
use tokio::sync::mpsc::UnboundedSender;

use crate::model::download_progress_item::DownloadProgressItem;
use crate::model::upload_progress_item::UploadProgressItem;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::types::{BucketLocationConstraint, CreateBucketConfiguration};
use aws_sdk_s3::{
    primitives::ByteStream,
    Client,
};
use aws_sdk_s3::error::ProvideErrorMetadata;
use color_eyre::{eyre, Report};
use crate::repositories::account_repo::get_default_account;
use crate::utils::CURRENT_ACCOUNT;

/// Handles interactions with the s3 services through AWS sdk
#[derive(Clone)]
pub struct S3DataFetcher {
    pub default_region: String,
    credentials: Credentials,
}

/*
- Handle buckets from different regions
- fix upload/download functions to handled dirs/buckets
- add create/delete buckets
- add copy files within a bucket
 */

impl S3DataFetcher {
    pub fn from_db_account() -> Option<Self> {
        if let Some(acc) = get_default_account() {
            let credentials = Credentials::new(
                acc.access_key,
                acc.secret_key,
                None,
                None,
                "db_account",
            );
            let default_region = acc.default_region;
            Some(S3DataFetcher {
                default_region,
                credentials,
            })
        } else {
            None
        }
    }

    /*
    this function handles only simple files as of now.
    - not sure when and if necessary to use multipart uploads,
    - no directory handling
     */
    pub async fn upload_item(
        &self,
        item: LocalSelectedItem,
        upload_tx: UnboundedSender<UploadProgressItem>,
    ) -> eyre::Result<bool> {
        let account = CURRENT_ACCOUNT.read().clone();
        let client = self.get_s3_client_with_account(account).await;
        let body = ByteStream::read_from()
            .path(item.path)
            // https://github.com/awslabs/aws-sdk-rust/blob/main/examples/examples/s3/src/bin/put-object-progress.rs
            // Artificially limit the buffer size to ensure the file has multiple
            // progress steps.
            // .buffer_size(2048)
            .build()
            .await?;
        let key = if item.destination_path == "/" {
            item.name
        } else {
            item.destination_path
        }; //Self::combine_paths(Path::new(&item.destination_path), Path::new(&item.name));
        //destination_path
        let request = client
            .put_object()
            .bucket(item.destination_bucket)
            .key(key)
            .body(body);

        // let customized = request
        //     .customize()
        //     .map_request(move |req| ProgressBody::<SdkBody>::replace(req, upload_tx.clone()));

        match request.send().await {
            Ok(_a) => Ok(true),
            Err(e) => {
                // println!("Upload SdkError: {:?}", e);
                Err(Report::msg(e.into_service_error().to_string()))
            }
        }
    }

    fn create_directory_structure(&self, full_path: &Path) -> eyre::Result<()> {
        // Extract the directory path
        if let Some(parent_dir) = full_path.parent() {
            // Create the directory structure
            fs::create_dir_all(parent_dir)?;
        }

        Ok(())
    }
    /*
    this function handles only simple files as of now.
    - no directory or full bucket handling
    */
    pub async fn download_item(
        &self,
        item: S3SelectedItem,
        download_tx: UnboundedSender<DownloadProgressItem>,
    ) -> eyre::Result<bool> {
        let account = CURRENT_ACCOUNT.read().clone();
        let client = self.get_s3_client_with_account(account).await;
        let mut path = PathBuf::from(item.destination_dir);
        path.push(item.path.clone().unwrap_or(item.name.clone()));
        self.create_directory_structure(&path)?;
        let mut file = File::create(&path)?;
        let bucket = item.bucket.expect("bucket must be defined").clone();
        let head_obj = client
            .head_object()
            .bucket(bucket.clone())
            .key(item.path.clone().unwrap_or(item.name.clone()))
            .send()
            .await?;
        match client
            .get_object()
            .bucket(bucket.clone())
            .key(item.path.clone().unwrap_or(item.name.clone()))
            .send()
            .await
        {
            Ok(mut object) => {
                let mut byte_count = 0_usize;
                let total = head_obj.content_length.unwrap_or(0i64);
                while let Some(bytes) = object.body.try_next().await? {
                    let bytes_len = bytes.len();
                    file.write_all(&bytes)?;
                    byte_count += bytes_len;
                    let progress = Self::calculate_download_percentage(total, byte_count);
                    let download_progress_item = DownloadProgressItem {
                        name: item.path.clone().unwrap_or(item.name.clone()),
                        bucket: bucket.clone(),
                        progress,
                    };
                    let _ = download_tx.send(download_progress_item);
                }
                Ok(true)
            }
            Err(e) => {
                // println!("Download SdkError: {:?}", e);
                Err(Report::msg(e.into_service_error().to_string()))
            }
        }
    }

    fn calculate_download_percentage(total: i64, byte_count: usize) -> f64 {
        if total == 0 {
            0.0 // Return 0% if total is 0 to avoid division by zero
        } else {
            (byte_count as f64 / total as f64) * 100.0 // Cast to f64 to ensure floating-point division
        }
    }

    pub async fn list_current_location(
        &self,
        bucket: Option<String>,
        prefix: Option<String>,
    ) -> eyre::Result<Vec<S3DataItem>> {
        println!("list_current_location");
        match (bucket, prefix) {
            (None, None) => self.list_all_buckets().await,
            (Some(bucket), None) => self.list_objects(bucket.as_str(), None).await,
            (Some(bucket), Some(prefix)) => self.list_objects(bucket.as_str(), Some(prefix)).await,
            _ => self.list_all_buckets().await,
        }
    }

    pub async fn get_bucket_location(&self, bucket: &str) -> eyre::Result<String> {
        let default_region = self.default_region.clone();
        let account = CURRENT_ACCOUNT.read().clone();
        let client = self.get_s3_client_with_account(account).await;
        
        println!("🔍 Getting location for bucket: '{}'", bucket);
        println!("🌍 Default region: '{}'", default_region);
        
        let head_obj = client.get_bucket_location().bucket(bucket).send().await?;
        
        println!("📡 AWS response for bucket '{}': {:?}", bucket, head_obj);
        
        let location_constraint = head_obj.location_constraint();
        println!("🗺️  Location constraint for bucket '{}': {:?}", bucket, location_constraint);
        
        let location = location_constraint
            .map(|lc| {
                let region_str = lc.to_string();
                println!("🔄 Converted location constraint to string: '{}'", region_str);
                region_str
            })
            .unwrap_or_else(|| {
                println!("⚠️  No location constraint found, using default region: '{}'", default_region);
                default_region.to_string()
            });
            
        println!("✅ Final region for bucket '{}': '{}'", bucket, location);
        Ok(location)
    }

    // Example async method to fetch data from an external service with pagination
    pub async fn list_buckets(&self, page: Option<usize>, page_size: Option<usize>) -> eyre::Result<(Vec<S3DataItem>, usize)> {
        let account = CURRENT_ACCOUNT.read().clone();
        let client = self.get_s3_client_with_account(account).await;
        let mut fetched_data: Vec<S3DataItem> = vec![];
        
        if let Ok(res) = client.list_buckets().send().await {
            fetched_data = res.buckets.as_ref().map_or_else(
                Vec::new, // In case there is no buckets field (it's None), return an empty Vec
                |buckets| {
                    buckets
                        .iter()
                        .filter_map(|bucket| {
                            // Filter out buckets where name is None, and map those with a name to a Vec<String>
                            bucket.name.as_ref().map(|name| {
                                let file_info = FileInfo {
                                    file_name: name.clone(),
                                    size: "".to_string(),
                                    file_type: "Bucket".to_string(),
                                    path: name.clone(),
                                    is_directory: false,
                                };
                                let bucket_info = BucketInfo {
                                    bucket: None,
                                    region: None,
                                    is_bucket: true,
                                };
                                S3DataItem::init(bucket_info, file_info)
                            })
                        })
                        .collect()
                },
            )
        }

        // Apply client-side pagination
        let total_count = fetched_data.len();
        
        if let (Some(page_num), Some(size)) = (page, page_size) {
            let start_index = page_num * size;
            let end_index = std::cmp::min(start_index + size, total_count);
            
            if start_index < total_count {
                fetched_data = fetched_data[start_index..end_index].to_vec();
            } else {
                fetched_data = vec![]; // Page beyond available data
            }
        }
        
        Ok((fetched_data, total_count))
    }

    // Convenience method to get all buckets (maintains backward compatibility)
    pub async fn list_all_buckets(&self) -> eyre::Result<Vec<S3DataItem>> {
        let (buckets, _) = self.list_buckets(None, None).await?;
        Ok(buckets)
    }

    pub async fn create_bucket(
        &self,
        name: String,
        region: String,
    ) -> eyre::Result<Option<String>> {
        let account = CURRENT_ACCOUNT.read().clone();
        let client = self.get_s3_client_with_account(account).await;
        let constraint = BucketLocationConstraint::from(region.as_str());
        let cfg = CreateBucketConfiguration::builder()
            .location_constraint(constraint)
            .build();
        match client
            .create_bucket()
            .create_bucket_configuration(cfg)
            .bucket(name.clone())
            .send()
            .await
        {
            Ok(_) => {
                // println!("Bucket created");
                Ok(None)
            }
            Err(e) => {
                println!("Cannot create bucket");
                Ok(Some(
                    e.into_service_error()
                        .message()
                        .unwrap_or("Cannot create bucket")
                        .to_string(),
                ))
            }
        }
    }

    pub async fn delete_data(
        &self,
        is_bucket: bool,
        bucket: Option<String>,
        name: String,
        _is_directory: bool,
    ) -> eyre::Result<Option<String>> {
        if is_bucket {
            let location = self.get_bucket_location(&name).await?;
            let creds = self.credentials.clone();
            let temp_file_creds = FileCredential {
                name: "temp".to_string(),
                access_key: creds.access_key_id().to_string(),
                secret_key: creds.secret_access_key().to_string(),
                default_region: location.clone(),
                selected: false,
            };
            let account = CURRENT_ACCOUNT.read().clone();
            let client_with_location = self.get_s3_client_with_account(account).await;
            let response = client_with_location
                .delete_bucket()
                .bucket(name.clone())
                .send()
                .await;
            match response {
                Ok(_) => {
                    println!("bucket deleted: {}", name);
                    Ok(None)
                }
                Err(e) => {
                    println!("error deleting bucket: {}, {:?}", name, e);
                    Ok(Some(
                        e.into_service_error()
                            .message()
                            .unwrap_or("Error deleting bucket")
                            .to_string(),
                    ))
                }
            }
        } else {
            println!("Deleting object: {:?}, {:?}", name, bucket);
            match bucket {
                Some(b) => self.delete_single_item(&b, &name).await,
                None => Ok(Some("No bucket specified!".into())),
            }
        }
    }

    async fn delete_single_item(&self, bucket: &str, name: &str) -> eyre::Result<Option<String>> {
        let location = self.get_bucket_location(bucket).await?;
        let creds = self.credentials.clone();
        let temp_file_creds = FileCredential {
            name: "temp".to_string(),
            access_key: creds.access_key_id().to_string(),
            secret_key: creds.secret_access_key().to_string(),
            default_region: location.clone(),
            selected: false,
        };
        let account = CURRENT_ACCOUNT.read().clone();
        let client_with_location = self.get_s3_client_with_account(account).await;
        let response = client_with_location
            .delete_object()
            .key(name)
            .bucket(bucket)
            .send()
            .await;
        match response {
            Ok(_) => {
                println!("S3 Object deleted, bucket: {:?}, name: {:?}", bucket, name);
                Ok(None)
            }
            Err(e) => {
                println!(
                    "Cannot delete object, bucket: {:?}, name: {:?}, error: {:?}",
                    bucket,
                    name,
                    e
                );
                Ok(Some(format!(
                    "Cannot delete object, {:?}",
                    e.into_service_error().message().unwrap_or("")
                )))
            }
        }
    }

    /// Lists all object in the given bucket (or filtered by prefix) and constructs the items
    /// representing directories
    /// This method is used for displaying bucket/prefix content while browsing s3 and
    /// it's not fetching all the contents behind prefixes together
    async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<String>,
    ) -> eyre::Result<Vec<S3DataItem>> {
        let mut all_objects = Vec::new();
        let location = self.get_bucket_location(bucket).await?;
        let creds = self.credentials.clone();
        let temp_file_creds = FileCredential {
            name: "temp".to_string(),
            access_key: creds.access_key_id().to_string(),
            secret_key: creds.secret_access_key().to_string(),
            default_region: location.clone(),
            selected: false,
        };
        let account = CURRENT_ACCOUNT.read().clone();
        let client_with_location = self.get_s3_client_with_account(account).await;
        let mut response = client_with_location
            .list_objects_v2()
            .delimiter("/")
            .set_prefix(prefix)
            .bucket(bucket.to_owned())
            .into_paginator()
            .send();

        while let Some(result) = response.next().await {
            match result {
                Ok(output) => {
                    for object in output.contents() {
                        let key = object.key().unwrap_or_default();
                        //todo: get size of the file
                        let size = object
                            .size()
                            .map_or(String::new(), |value| value.to_string());
                        let path = Path::new(key);
                        let file_extension = path
                            .extension()
                            .and_then(|ext| ext.to_str()) // Convert the OsStr to a &str
                            .unwrap_or("");
                        let file_info = FileInfo {
                            file_name: Self::get_filename(key).unwrap_or_default(),
                            size,
                            file_type: file_extension.to_string(),
                            path: key.to_string(),
                            is_directory: false,
                        };
                        let bucket_info = BucketInfo {
                            bucket: Some(bucket.to_string()),
                            region: Some(location.clone()),
                            is_bucket: false,
                        };
                        all_objects.push(S3DataItem::init(bucket_info, file_info));
                    }
                    for object in output.common_prefixes() {
                        let key = object.prefix().unwrap_or_default();
                        if key != "/" {
                            let file_info = FileInfo {
                                file_name: Self::get_last_directory(key).unwrap_or_default(),
                                size: "".to_string(),
                                file_type: "Dir".to_string(),
                                path: key.to_string(),
                                is_directory: true,
                            };
                            let bucket_info = BucketInfo {
                                bucket: Some(bucket.to_string()),
                                region: Some(location.clone()),
                                is_bucket: false,
                            };
                            all_objects.push(S3DataItem::init(bucket_info, file_info));
                        }
                    }
                }
                Err(err) => {
                    println!("Err: {:?}", err) // Return the error immediately if encountered
                }
            }
        }

        Ok(all_objects)
    }

    fn get_last_directory(path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('/').collect();
        let parts: Vec<&str> = parts.into_iter().filter(|&part| !part.is_empty()).collect();
        parts.last().map(|&last| format!("{}/", last))
    }
    fn get_filename(path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('/').collect();
        let parts: Vec<&str> = parts.into_iter().filter(|&part| !part.is_empty()).collect();
        parts.last().and_then(|&last| {
            if path.ends_with('/') {
                None
            } else {
                Some(last.to_string())
            }
        })
    }

    /// This method is similar to `list_objects` but it fetches all the data recursively
    /// including data behind the prefixes.
    /// Designed to be used mainly when selecting whole bucket/prefix for download or delete.
    pub async fn list_all_objects(
        &self,
        bucket: &str,
        prefix: Option<String>,
    ) -> eyre::Result<Vec<S3DataItem>> {
        let mut all_objects = Vec::new();
        let location = self.get_bucket_location(bucket).await?;
        self.recursive_list_objects(bucket, prefix, &location, &mut all_objects)
            .await?;
        Ok(all_objects)
    }
    fn recursive_list_objects<'a>(
        &'a self,
        bucket: &'a str,
        prefix: Option<String>,
        location: &'a str,
        all_objects: &'a mut Vec<S3DataItem>,
    ) -> Pin<Box<dyn std::future::Future<Output=Result<(), Report>> + Send + 'a>> {
        let account = CURRENT_ACCOUNT.read().clone();
        Box::pin(async move {
            let creds = self.credentials.clone();
            let temp_file_creds = FileCredential {
                name: "temp".to_string(),
                access_key: creds.access_key_id().to_string(),
                secret_key: creds.secret_access_key().to_string(),
                default_region: location.to_string(),
                selected: false,
            };
            let client_with_location = self.get_s3_client_with_account(account).await;
            let mut response = client_with_location
                .list_objects_v2()
                .delimiter("/")
                .set_prefix(prefix.clone())
                .bucket(bucket.to_owned())
                .into_paginator()
                .send();

            while let Some(result) = response.next().await {
                match result {
                    Ok(output) => {
                        for object in output.contents() {
                            let key = object.key().unwrap_or_default();
                            let size = object
                                .size()
                                .map_or(String::new(), |value| value.to_string());
                            let path = Path::new(key);
                            let file_extension =
                                path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                            let file_info = FileInfo {
                                file_name: Self::get_filename(key).unwrap_or_default(),
                                size,
                                file_type: file_extension.to_string(),
                                path: key.to_string(),
                                is_directory: false,
                            };
                            let bucket_info = BucketInfo {
                                bucket: Some(bucket.to_string()),
                                region: Some(location.to_string()),
                                is_bucket: false,
                            };
                            all_objects.push(S3DataItem::init(bucket_info, file_info));
                        }
                        for common_prefix in output.common_prefixes() {
                            let prefix = common_prefix.prefix().unwrap_or_default().to_string();
                            self.recursive_list_objects(
                                bucket,
                                Some(prefix),
                                location,
                                all_objects,
                            )
                                .await?;
                        }
                    }
                    Err(err) => {
                        println!("Err: {:?}", err); // Return the error immediately if encountered
                        return Err(err.into());
                    }
                }
            }
            Ok(())
        })
    }

    async fn get_s3_client(&self, creds: Option<FileCredential>) -> Client {
        let credentials: Credentials;
        let default_region: String;
        if let Some(crd) = creds {
            let access_key = crd.access_key;
            let secret_access_key = crd.secret_key;
            default_region = crd.default_region;
            credentials = Credentials::new(
                access_key,
                secret_access_key,
                None,     // Token, if using temporary credentials (like STS)
                None,     // Expiry time, if applicable
                "manual", // Source, just a label for debugging
            );
        } else {
            credentials = self.credentials.clone();
            default_region = self.default_region.clone();
        }
        let region_provider = RegionProviderChain::first_try(Region::new(default_region))
            .or_default_provider()
            .or_else(Region::new("eu-north-1"));
        let shared_config = aws_config::from_env()
            .credentials_provider(credentials)
            .region(region_provider)
            .load()
            .await;
        Client::new(&shared_config)
    }

    /// Like get_s3_client, but takes Option<Account> (from DB or given), not FileCredential.
    pub async fn get_s3_client_with_account(&self, account: Option<crate::model::account::Account>) -> Client {
        let credentials: Credentials;
        let default_region: String;
        if let Some(acc) = account {
            let access_key = acc.access_key;
            let secret_access_key = acc.secret_key;
            default_region = self.default_region.clone();
            credentials = Credentials::new(
                access_key,
                secret_access_key,
                None,     // Token, if using temporary credentials (like STS)
                None,     // Expiry time, if applicable
                "account_struct", // Source, just a label for debugging
            );
        } else {
            if let Some(acc) = get_default_account() {
                credentials = Credentials::new(
                    acc.access_key,
                    acc.secret_key,
                    None,
                    None,
                    "db_account",
                );
                default_region = self.default_region.clone();
            } else {
                // fallback to self.credentials if no account found
                credentials = self.credentials.clone();
                default_region = self.default_region.clone();
            }
        }
        let region_provider = RegionProviderChain::first_try(Region::new(default_region))
            .or_default_provider()
            .or_else(Region::new("eu-north-1"));
        let shared_config = aws_config::from_env()
            .credentials_provider(credentials)
            .region(region_provider)
            .load()
            .await;
        Client::new(&shared_config)
    }
}
