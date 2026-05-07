use std::{marker::PhantomData, num::NonZero};

use crate::{ObjectID, data_definitions::id::ID};
use chrono::{DateTime, Local};
use serde::Serialize;
use sqlx::MySql;

#[derive(PartialEq, Clone, Copy, Serialize)]
pub(crate) enum State {
    Created,
    Deleted,
    Pending,
}


impl TryFrom<u16> for State {
    type Error = sqlx::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Created),
            2 => Ok(Self::Deleted),
            3 => Ok(Self::Pending),
            _ => Err(sqlx::Error::Protocol(
                "Invalid conversion, check database schema".into(),
            )),
        }
    }
}

impl From<State> for u16 {
    fn from(value: State) -> Self {
        match value {
            State::Created => 1,
            State::Deleted => 2,
            State::Pending => 3,
        }
    }
}

impl sqlx::Decode<'_, MySql> for State {
    fn decode(
        value: <MySql as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        match u16::decode(value)?.try_into() {
            Ok(state) => Ok(state),
            Err(err) => Err(Box::new(err)),
        }
    }
}

impl sqlx::Encode<'_, MySql> for State {
    fn encode(
        self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        u16::encode(self.into(), buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        u16::encode_by_ref(&(*self).into(), buf)
    }
}

impl sqlx::Type<MySql> for State {
    fn type_info() -> <MySql as sqlx::Database>::TypeInfo {
        u16::type_info()
    }
}

pub(crate) struct File;
pub(crate) struct Directory;

pub(crate) trait Kind {
    const IS_FOLDER: bool;
    fn size_bytes(size: Option<NonZero<u64>>) -> Option<NonZero<u64>>;
}

impl Kind for File {
    const IS_FOLDER: bool = false;
    fn size_bytes(size: Option<NonZero<u64>>) -> Option<NonZero<u64>> {
        size
    }
}

impl Kind for Directory {
    const IS_FOLDER: bool = true;
    fn size_bytes(size: Option<NonZero<u64>>) -> Option<NonZero<u64>> {
        if size.is_some() {
            panic!("Invalid state never do that")
        }
        None
    }
}

pub trait Descriptor {
    fn serialize(&self) -> impl Serialize;
}

pub(crate) struct EntryDraft<'a, Kind> {
    user_id: ID,
    name: &'a str,
    size_bytes: Option<NonZero<u64>>, // Allows for Null writes on the DB
    parent_id: Option<ObjectID>,
    state: State,

    _marker: PhantomData<Kind>,
}

impl<'a, Kind> EntryDraft<'a, Kind> {
    pub(crate) const fn get_user(&self) -> ID {
        self.user_id
    }

    pub(crate) const fn get_name(&self) -> &str {
        self.name
    }

    pub(crate) const fn get_size_bytes(&self) -> Option<NonZero<u64>> {
        self.size_bytes
    }

    pub(crate) const fn get_parent(&self) -> Option<ObjectID> {
        self.parent_id
    }

    pub(crate) const fn get_state(&self) -> State {
        self.state
    }
}

pub(crate) struct EntryPersisted<Kind> {
    id: ObjectID,
    user_id: ID,
    name: String,
    size_bytes: Option<NonZero<u64>>,
    parent_id: Option<ObjectID>,
    state: State,
    created_at: DateTime<Local>,
    modified_at: DateTime<Local>,

    _marker: PhantomData<Kind>,
}

impl<'a, K: Kind> EntryDraft<'a, K> {
    pub fn new(
        user_id: ID,
        name: &'a str,
        size_bytes: Option<NonZero<u64>>,
        parent_id: Option<ObjectID>,
        state: State,
    ) -> Self {
        Self {
            user_id,
            name,
            size_bytes: K::size_bytes(size_bytes),
            parent_id,
            state,
            _marker: PhantomData,
        }
    }
}

impl<K: Kind> EntryPersisted<K> {
    pub fn new(
        base: EntryDraft<'_, K>,
        object_id: ObjectID,
        created_at: DateTime<Local>,
        modified_at: DateTime<Local>,
    ) -> Self {
        Self {
            id: object_id,
            user_id: base.user_id,
            name: base.name.to_owned(),
            size_bytes: base.size_bytes,
            parent_id: base.parent_id,
            state: base.state,
            created_at: created_at,
            modified_at: modified_at,
            _marker: PhantomData,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct FileDescriptor {
    id: ObjectID,
    user_id: ID,
    size_bytes: u64,
    parent_id: Option<ObjectID>,
    state: State,
    is_folder: bool,
    created_at: DateTime<Local>,
    modified_at: DateTime<Local>,
}

impl<K: Kind> Descriptor for EntryPersisted<K> {
    fn serialize(&self) -> impl Serialize {
        FileDescriptor {
            id: self.id,
            user_id: self.user_id,
            size_bytes: self.size_bytes.map_or(0, |inner| inner.get()),
            parent_id: self.parent_id,
            state: self.state,
            is_folder: K::IS_FOLDER,
            created_at: self.created_at,
            modified_at: self.modified_at,
        }
    }
}
