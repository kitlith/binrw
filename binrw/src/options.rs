use crate::Endian;
use typemap_core::{Contains, TypeMapGet};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct FileOffset(pub u64);

// TODO: u32, u16, u8?
impl From<u64> for FileOffset {
    fn from(val: u64) -> Self {
        FileOffset(val)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DontOutputTemplate(pub bool);
#[derive(Debug, PartialEq, Clone, Default)]
pub struct VariableName(pub Option<&'static str>);

pub trait ReadOptionsExt {
    fn endian(&self) -> Endian where Self: Contains<Endian>;
    fn offset(&self) -> u64 where Self: Contains<FileOffset>;
    #[cfg(feature = "debug_template")]
    fn dont_output_to_template(&self) -> bool;
    #[cfg(feature = "debug_template")]
    fn variable_name(&self) -> Option<&'static str>;
}

impl<T: TypeMapGet> ReadOptionsExt for T {
    fn endian(&self) -> Endian
    where
        Self: Contains<Endian>,
    {
        *self.get::<Endian>()
    }

    fn offset(&self) -> u64
    where
        Self: Contains<FileOffset>,
    {
        self.get::<FileOffset>().0
    }

    #[cfg(feature = "debug_template")]
    fn dont_output_to_template(&self) -> bool {
        self.try_get::<DontOutputTemplate>()
            .map(|a| a.0)
            .unwrap_or(false)
    }

    #[cfg(feature = "debug_template")]
    fn variable_name(&self) -> Option<&'static str> {
        self.try_get::<VariableName>().and_then(|a| a.0)
    }
}
