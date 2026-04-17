use uuid::Uuid;

use crate::database::{Transactional, virtual_filesystem::create_file::CreateFile};

pub struct VirtualFileSystem;

impl VirtualFileSystem {
    pub fn create_file(
        id: Uuid,
        user_id: i32,
        name: &str,
        size_bytes: u64,
        parent_id: Option<Uuid>,
        is_folder: bool,
    ) -> impl Transactional {
        CreateFile::new(id, user_id, name, size_bytes, parent_id, is_folder)
    }
}

mod create_file;
