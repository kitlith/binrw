use crate::Endian;
use crate::options::Options;

/// Runtime-configured options for reading the type using [`BinRead`](BinRead)
#[non_exhaustive]
#[derive(Default, Clone, Copy)]
pub struct ReadOptions {
    pub endian: Endian,
    pub count: Option<usize>,
    pub offset: u64,

    #[cfg(feature = "debug_template")]
    pub dont_output_to_template: bool,
    #[cfg(feature = "debug_template")]
    pub variable_name: Option<&'static str>,
}

pub struct VecCount(pub usize);
pub struct FileOffset(pub u64);

#[cfg(feature = "debug_template")]
pub struct DontOutputTemplate;
#[cfg(feature = "debug_template")]
pub struct VariableName(pub &'static str);

pub trait ReadOptionsExt {
    fn endian(&self) -> Endian;
    fn count(&self) -> Option<usize>;
    fn offset(&self) -> u64;

    #[cfg(feature = "debug_template")]
    fn dont_output_to_template(&self) -> bool;
    #[cfg(feature = "debug_template")]
    fn variable_name(&self) -> Option<&'static str>;
}

impl ReadOptionsExt for Options {
    fn endian(&self) -> Endian {
        *self.get::<Endian>().unwrap_or(&Endian::default())
    }

    fn count(&self) -> Option<usize> {
        self.get::<VecCount>().map(|x| x.0)
    }

    fn offset(&self) -> u64 {
        self.get::<FileOffset>().unwrap_or(&FileOffset(0)).0
    }

    #[cfg(feature = "debug_template")]
    fn dont_output_to_template(&self) -> bool {
        self.contains::<DontOutputTemplate>()
    }

    #[cfg(feature = "debug_template")]
    fn variable_name(&self) -> Option<&'static str> {
        self.get::<VariableName>().map(|x| x.0)
    }
}