use rocket::tokio::io::{AsyncBufRead, AsyncRead};

pub trait Storage: Send + Sync {
    type Success: ObjectIdentifier;
    type Error;
    type ResourceHandle: AsyncBufRead;

    fn store(&self, object: &mut (dyn AsyncRead + Unpin)) -> impl Future<Output = Result<Self::Success, Self::Error>> + Send;
    fn retrieve(&self, object: &dyn ObjectIdentifier) -> impl Future<Output = Result<Self::ResourceHandle, Self::Error>> + Send;
}

type UUID = [u8; 16];

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub (crate) struct ObjectID(UUID);

pub (crate) trait ObjectIdentifier {
    fn get_id(&self) -> ObjectID;
}
