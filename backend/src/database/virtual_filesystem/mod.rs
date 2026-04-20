use uuid::Uuid;

use crate::database::{
    ReadOnly, Transactional,
    virtual_filesystem::{
        create_file::CreateFile, create_folder::CreateFolder, list_files::ListFiles,
    },
};

pub (crate) use list_files::FileRow;

pub struct VirtualFileSystem;

impl VirtualFileSystem {
    pub fn create_file(
        id: Uuid,
        user_id: i32,
        name: &str,
        size_bytes: u64,
        parent_id: Option<Uuid>,
    ) -> impl Transactional<Success = (), Error = sqlx::Error> {
        CreateFile::new(id, user_id, name, size_bytes, parent_id)
    }

    pub fn create_folder(
        id: Uuid,
        user_id: i32,
        name: &str,
        parent_id: Option<Uuid>,
    ) -> impl Transactional<Success = (), Error = sqlx::Error> {
        CreateFolder::new(id, user_id, name, parent_id)
    }

    pub fn list_files(
        user_id: i32,
        parent_id: Option<Uuid>,
    ) -> impl ReadOnly<Success = Vec<FileRow>, Error = sqlx::Error> {
        ListFiles::new(user_id, parent_id)
    }
}

mod create_file;
mod create_folder;
mod list_files;
