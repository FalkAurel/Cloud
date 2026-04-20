use uuid::Uuid;

use crate::database::{
    ReadOnly, Transactional,
    virtual_filesystem::{
        create_file::CreateFile, create_folder::CreateFolder, get_file::GetFile,
        list_files::ListFiles,
    },
};

pub(crate) use get_file::FileEntry;
pub(crate) use list_files::FileRow;

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

    pub fn get_file(
        file_id: Uuid,
        user_id: i32,
    ) -> impl ReadOnly<Success = Option<FileEntry>, Error = sqlx::Error> {
        GetFile::new(file_id, user_id)
    }
}

mod create_file;
mod create_folder;
mod get_file;
mod list_files;
