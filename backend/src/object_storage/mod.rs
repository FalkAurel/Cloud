use rocket::tokio::io::{AsyncBufRead, AsyncRead};
use std::{error::Error, future::Future, pin::Pin};
use uuid::Uuid;

pub trait Storage: Send + Sync {
    fn store<'b>(
        &'b self,
        object: &'b mut (dyn AsyncRead + Unpin + Send + 'b),
    ) -> Pin<Box<dyn Future<Output = Result<ObjectID, Box<dyn Error + Send>>> + Send + 'b>>;

    fn retrieve<'b>(
        &'b self,
        object: ObjectID,
    ) -> Pin<
        Box<dyn Future<Output = Result<Box<dyn AsyncBufRead>, Box<dyn Error + Send>>> + Send + 'b>,
    >;

    fn delete<'b>(
        &'b self,
        object: ObjectID,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send>>> + Send + 'b>>;
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct ObjectID(pub(crate) Uuid);

mod s3;
pub use s3::S3StorageDevice;

#[cfg(test)]
pub(crate) mod mock_storage {
    use crate::{ObjectID, Storage};
    use std::{error::Error, fmt};
    use uuid::Uuid;

    pub(crate) struct MockStorage<const IS_ALWAYS_SUCCESSFUL: bool>;

    #[derive(Debug)]
    pub(crate) struct MockStorageAsyncReader;

    impl rocket::tokio::io::AsyncRead for MockStorageAsyncReader {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            _buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            panic!("This is only mock device. You are not supposed to consume it!")
        }
    }

    impl rocket::tokio::io::AsyncBufRead for MockStorageAsyncReader {
        fn consume(self: std::pin::Pin<&mut Self>, _amt: usize) {
            panic!("This is only mock device. You are not supposed to consume it!")
        }

        fn poll_fill_buf(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<&[u8]>> {
            panic!("This is only mock device. You are not supposed to consume it!")
        }
    }

    #[derive(Debug)]
    pub(crate) struct MockStorageError;

    impl fmt::Display for MockStorageError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Just MockError")
        }
    }

    impl Error for MockStorageError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }

    impl<const IS_ALWAYS_SUCCESSFUL: bool> Storage for MockStorage<IS_ALWAYS_SUCCESSFUL> {
        fn store<'b>(
            &'b self,
            _object: &'b mut (dyn tokio::io::AsyncRead + Unpin + Send + 'b),
        ) -> std::pin::Pin<
            Box<dyn Future<Output = Result<super::ObjectID, Box<dyn Error + Send>>> + Send + 'b>,
        > {
            Box::pin(async {
                if IS_ALWAYS_SUCCESSFUL {
                    Ok(ObjectID(Uuid::new_v4()))
                } else {
                    Err(Box::new(MockStorageError) as Box<dyn Error + Send>)
                }
            })
        }

        fn delete<'b>(
            &'b self,
            _object: ObjectID,
        ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send>>> + Send + 'b>>
        {
            Box::pin(async {
                if IS_ALWAYS_SUCCESSFUL {
                    Ok(())
                } else {
                    Err(Box::new(MockStorageError) as Box<dyn Error + Send>)
                }
            })
        }

        fn retrieve<'b>(
            &'b self,
            _object: ObjectID,
        ) -> std::pin::Pin<
            Box<
                dyn Future<
                        Output = Result<
                            Box<dyn tokio::io::AsyncBufRead>,
                            Box<dyn std::error::Error + Send>,
                        >,
                    > + Send
                    + 'b,
            >,
        > {
            Box::pin(async {
                if IS_ALWAYS_SUCCESSFUL {
                    Ok(Box::new(MockStorageAsyncReader)
                        as Box<dyn rocket::tokio::io::AsyncBufRead>)
                } else {
                    Err(Box::new(MockStorageError) as Box<dyn Error + Send>)
                }
            })
        }
    }
}
