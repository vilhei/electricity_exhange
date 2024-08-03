use core::{convert::From, str::FromStr};

use embassy_embedded_hal::adapter::BlockingAsync;
use esp_storage::FlashStorage;
use heapless::{String, Vec};
use postcard::experimental::max_size::MaxSize;
use sequential_storage::{
    cache::NoCache,
    map::{fetch_item, store_item, SerializationError},
};
use serde::{Deserialize, Serialize};

pub enum StorageMessage {
    Fetch,
    Store,
}

/// Abstract access to NVS (Non Volatile Storage).
///
/// Data in NVS persists over reboots.
///
/// Uses [sequential_storage] together with [esp_storage] to access NVS
///
/// NVS is located at 0x9000 and is either 0x4000 or 0x6000 long (not sure so I use 0x4000).
pub struct NonVolatileStorage {
    flash: BlockingAsync<FlashStorage>,
}

impl NonVolatileStorage {
    const FLASH_RANGE: core::ops::Range<u32> = 0x9000..(0x9000 + 0x4000);

    /// Returns instance of [NonVolatileStorage] **once**.
    ///
    /// If called multiple times will panic
    pub fn take() -> Self {
        static mut _NON_VOLATILE_STORAGE_TAKEN: bool = false;
        // println!("ITEM SIZE : {ITEM_SIZE}");
        critical_section::with(|_| unsafe {
            if _NON_VOLATILE_STORAGE_TAKEN {
                panic!("NonVolatileStorage already taken");
                _NON_VOLATILE_STORAGE_TAKEN = true;
            }
            Self::steal()
        })
    }

    /// Unsafely creates an instance of [NonVolatileStorage]
    ///
    /// # Safety
    /// You really should only use one instance of this type.
    pub unsafe fn steal() -> Self {
        // Use BlockingASync wrapper because esp_storage::FlashStorage does not implement embedded_storage_async::nor_flash::NorFlash trait
        let flash = BlockingAsync::new(esp_storage::FlashStorage::new());
        Self { flash }
    }

    /// Returns [NonVolatileItem] if one can be found with given [NonVolatileKey].
    ///
    /// Returns [None] if there are no errors but the item with the given key does not exist
    ///
    /// # Errors
    ///
    /// This function will return an error if the item with given key can not be retrieved,
    /// this can be because of serialization error, corrupted memory, flash error etc.
    pub async fn fetch(
        &mut self,
        key: NonVolatileKey,
    ) -> Result<Option<NonVolatileItem>, StorageError> {
        let mut data_buffer = [0u8; ITEM_SIZE];

        fetch_item(
            &mut self.flash,
            Self::FLASH_RANGE,
            &mut NoCache::new(),
            &mut data_buffer,
            key,
        )
        .await
        .map_err(|e| e.into())
    }

    /// Stores [NonVolatileItem] with [NonVolatileKey]
    ///
    /// This item can be later fetched using [Self::fetch]
    ///
    /// # Errors
    ///
    /// This function will return an error if the item with given key can not be stored,
    /// this can be because of serialization error, corrupted memory, flash error etc.
    pub async fn store(
        &mut self,
        key: NonVolatileKey,
        item: String<64>,
    ) -> Result<(), StorageError> {
        let mut data_buffer = [0u8; ITEM_SIZE];
        store_item(
            &mut self.flash,
            Self::FLASH_RANGE,
            &mut NoCache::new(),
            &mut data_buffer,
            key,
            &NonVolatileItem(item),
        )
        .await
        .map_err(|e| e.into())
    }
}

const ITEM_SIZE: usize = size_of::<NonVolatileItem>() + NonVolatileKey::POSTCARD_MAX_SIZE;

#[repr(C)]
pub struct NonVolatileItem(pub String<64>);

impl NonVolatileItem {
    pub fn new(item: &str) -> Self {
        Self(String::from_str(item).unwrap())
    }
}

impl<'a> sequential_storage::map::Value<'a> for NonVolatileItem {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        let len = self.0.len();
        buffer[..len].copy_from_slice(self.0.as_bytes());
        Ok(len)
    }

    fn deserialize_from(buffer: &'a [u8]) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let mut b = Vec::<u8, 64>::new();
        b.extend_from_slice(buffer).unwrap();
        let s = String::from_utf8(b).unwrap();
        Ok(NonVolatileItem(s))
    }
}

impl AsRef<str> for NonVolatileItem {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

/// Contains all the ids for possible stored data in the non volatile memory
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, MaxSize)]
#[repr(C)]
pub enum NonVolatileKey {
    WifiSsid,
    WifiPassword,
    FingridApiKey,
    EntsoeApiKey,
}

impl sequential_storage::map::Key for NonVolatileKey {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        let buf = postcard::to_slice(self, buffer).map_err(|e| match e {
            postcard::Error::SerializeBufferFull => SerializationError::BufferTooSmall,
            postcard::Error::DeserializeBadVarint
            | postcard::Error::DeserializeBadBool
            | postcard::Error::DeserializeBadChar
            | postcard::Error::DeserializeBadUtf8
            | postcard::Error::DeserializeBadOption
            | postcard::Error::DeserializeBadEnum
            | postcard::Error::DeserializeBadEncoding => SerializationError::InvalidData,
            _ => SerializationError::Custom(0),
        })?;
        Ok(buf.len())
    }

    fn deserialize_from(buffer: &[u8]) -> Result<(Self, usize), SerializationError> {
        let key: NonVolatileKey = postcard::from_bytes(buffer).map_err(|e| match e {
            postcard::Error::SerializeBufferFull => SerializationError::BufferTooSmall,
            postcard::Error::DeserializeBadVarint
            | postcard::Error::DeserializeBadBool
            | postcard::Error::DeserializeBadChar
            | postcard::Error::DeserializeBadUtf8
            | postcard::Error::DeserializeBadOption
            | postcard::Error::DeserializeBadEnum
            | postcard::Error::DeserializeBadEncoding => SerializationError::InvalidData,
            _ => SerializationError::Custom(0),
        })?;

        Ok((key, NonVolatileKey::POSTCARD_MAX_SIZE))
    }
}

#[derive(Debug)]
pub enum StorageError {
    /// An error in the storage (flash)
    Storage,
    /// Item can not be stored because storage is full
    FullStorage,
    /// Memory is likely corrupted, you may want to erase memory to recover
    Corrupted,
    /// Provided buffer was too big to be used
    BufferTooBig,
    /// Provided buffer was too small. Usize is the size needed
    BufferTooSmall(usize),
    /// Either key or value serialization error
    SerializationError,
    /// Item is too big to fit even to empty flash
    ItemTooBig,
}

impl From<sequential_storage::Error<esp_storage::FlashStorageError>> for StorageError {
    fn from(value: sequential_storage::Error<esp_storage::FlashStorageError>) -> Self {
        match value {
            sequential_storage::Error::Storage { .. } => Self::Storage,
            sequential_storage::Error::FullStorage => Self::FullStorage,
            sequential_storage::Error::Corrupted {} => Self::Corrupted,
            sequential_storage::Error::BufferTooBig => Self::BufferTooBig,
            sequential_storage::Error::BufferTooSmall(s) => Self::BufferTooSmall(s),
            sequential_storage::Error::SerializationError(_) => Self::SerializationError,
            sequential_storage::Error::ItemTooBig => Self::ItemTooBig,
            _ => todo!(),
        }
    }
}
