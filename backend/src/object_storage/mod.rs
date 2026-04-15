use rocket::tokio::io::{AsyncBufRead, AsyncRead};
use std::{error::Error, future::Future, pin::Pin};
use uuid::Uuid;

pub trait Storage: Send + Sync {
    fn store<'b>(
        &'b self,
        object: &'b mut (dyn AsyncRead + Unpin + Send + 'b),
    ) -> Pin<Box<dyn Future<Output = Result<ObjectID, Box<dyn Error>>> + Send + 'b>>;

    fn retrieve<'b>(
        &'b self,
        object: ObjectID,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn AsyncBufRead>, Box<dyn Error>>> + Send + 'b>>;
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct ObjectID(Uuid);

mod s3;
pub use s3::S3StorageDevice;
