use crate::{
    ObjectID,
    data_definitions::{
        file::{Directory, EntryDraft, File},
        id::ID,
    },
    database::{
        ReadOnly, Transactional,
        virtual_filesystem::{
            create_file::CreateFile, create_folder::CreateFolder, list_files::ListFiles,
        },
    },
};

pub(crate) use list_files::FileRow;

pub struct VirtualFileSystem;

impl VirtualFileSystem {
    pub fn create_file(
        file: &EntryDraft<'_, File>,
    ) -> impl Transactional<Success = ObjectID, Error = sqlx::Error> {
        CreateFile::new(file)
    }

    pub fn create_folder(
        folder: &EntryDraft<'_, Directory>,
    ) -> impl Transactional<Success = ObjectID, Error = sqlx::Error> {
        CreateFolder::new(folder)
    }

    pub fn list_files(
        user_id: ID,
        parent_id: Option<ObjectID>,
    ) -> impl ReadOnly<Success = Vec<FileRow>, Error = sqlx::Error> {
        ListFiles::new(user_id, parent_id)
    }
}

mod create_file;
mod create_folder;
mod list_files;
