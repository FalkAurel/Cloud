use std::{env, error::Error, future::Future, pin::Pin};

use crate::object_storage::{ObjectID, Storage};
use minio::s3::{
    Client, builders::ObjectContent, creds::StaticProvider, http::BaseUrl,
    response::BucketExistsResponse, types::S3Api,
};
use rocket::tokio::io::{AsyncBufRead, AsyncRead, AsyncReadExt};
use tokio_util::io::StreamReader;
use uuid::Uuid;

fn to_send_error(e: impl Error + Send + 'static) -> Box<dyn Error + Send> {
    Box::new(e)
}

pub struct S3StorageDevice {
    client: Client,
    bucket: String,
}

impl S3StorageDevice {
    pub async fn from_env() -> Self {
        let minio_user: String = env::var("MINIO_ROOT_USER").unwrap_or_else(|_| {
            tracing::error!("MINIO_ROOT_USER env var is not set");
            panic!("MINIO_ROOT_USER env var is not set");
        });
        let minio_password: String = env::var("MINIO_ROOT_PASSWORD").unwrap_or_else(|_| {
            tracing::error!("MINIO_ROOT_PASSWORD env var is not set");
            panic!("MINIO_ROOT_PASSWORD env var is not set");
        });
        let bucket_name: String = env::var("BUCKET_NAME").unwrap_or_else(|_| {
            tracing::error!("BUCKET_NAME env var is not set");
            panic!("BUCKET_NAME env var is not set");
        });

        let base_url: BaseUrl = "http://minio:9000".parse::<BaseUrl>().unwrap_or_else(|e| {
            tracing::error!(error = %e, "Failed to parse MinIO base URL");
            panic!("Failed to parse MinIO base URL: {e}");
        });

        let client: Client = Client::new(
            base_url,
            Some(Box::new(StaticProvider::new(
                &minio_user,
                &minio_password,
                None,
            ))),
            None,
            None,
        )
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "Failed to create MinIO client");
            panic!("Failed to create MinIO client: {e}");
        });

        Self::new(client, &bucket_name).await.unwrap_or_else(|e| {
            tracing::error!(error = %e, bucket = %bucket_name, "Failed to initialise S3 storage");
            panic!("Failed to initialise S3 storage: {e}");
        })
    }

    pub async fn new(client: Client, bucket_name: &str) -> Result<Self, Box<dyn Error>> {
        let exist_bucket: BucketExistsResponse = client.bucket_exists(bucket_name).send().await?;

        if !exist_bucket.exists {
            client.create_bucket(bucket_name).send().await?;
        }

        Ok(Self {
            client,
            bucket: bucket_name.to_owned(),
        })
    }
}

impl Storage for S3StorageDevice {
    fn store<'b>(
        &'b self,
        object: &'b mut (dyn AsyncRead + Unpin + Send + 'b),
    ) -> Pin<Box<dyn Future<Output = Result<ObjectID, Box<dyn Error + Send>>> + Send + 'b>> {
        Box::pin(async move {
            const CHUNK_SIZE: usize = 8 * 1024 * 1024; // 8 MiB — above S3 minimum multipart size
            let mut buffer: Vec<u8> = vec![0u8; CHUNK_SIZE];
            let uuid: Uuid = Uuid::new_v4();
            let object_name: String = uuid.to_string();

            loop {
                // Fill the buffer before uploading to minimise syscalls and network round trips
                let mut filled: usize = 0;
                loop {
                    match object.read(&mut buffer[filled..]).await {
                        Ok(0) => break,
                        Ok(n) => {
                            filled += n;
                            if filled == CHUNK_SIZE {
                                break;
                            }
                        }
                        Err(err) => return Err(to_send_error(err)),
                    }
                }

                if filled == 0 {
                    break;
                }

                if let Err(err) = self
                    .client
                    .append_object_content(
                        &self.bucket,
                        object_name.clone(),
                        ObjectContent::from(buffer[..filled].to_vec()),
                    )
                    .send()
                    .await
                {
                    return Err(to_send_error(err));
                }

                if filled < CHUNK_SIZE {
                    break; // EOF reached mid-fill, we're done
                }
            }

            Ok(ObjectID(uuid))
        })
    }

    fn retrieve<'b>(
        &'b self,
        object: ObjectID,
    ) -> Pin<
        Box<dyn Future<Output = Result<Box<dyn AsyncBufRead>, Box<dyn Error + Send>>> + Send + 'b>,
    > {
        let object_name: String = object.0.to_string();
        Box::pin(async move {
            match self
                .client
                .get_object(&self.bucket, object_name)
                .send()
                .await
            {
                Ok(resp) => {
                    let (stream, _) = resp.content.to_stream().await.map_err(to_send_error)?;
                    Ok(Box::new(StreamReader::new(stream)) as Box<dyn AsyncBufRead>)
                }
                Err(err) => Err(to_send_error(err)),
            }
        })
    }

    fn delete<'b>(
        &'b self,
        object: ObjectID,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send>>> + Send + 'b>> {
        let object_name: String = object.0.to_string();

        Box::pin(async {
            self.client
                .delete_object(&self.bucket, object_name)
                .send()
                .await
                .map_err(to_send_error)?;
            Ok(())
        })
    }
}
