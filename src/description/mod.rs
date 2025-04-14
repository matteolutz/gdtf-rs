use crate::description::util::IterUtil;
use crate::fixture_type::FixtureType;
use crate::validation::{ValidationError, ValidationErrorType, ValidationObject, ValidationResult};
use crate::values::{Name, Version};
use crate::{GdtfError, GdtfResult, ResourceMap};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::io::BufRead;
use std::str::FromStr;

#[macro_use]
mod collect_helper;
mod parse_helper;
mod util;

pub mod attribute;
pub mod dmx_mode;
pub mod fixture_type;
pub mod ft_preset;
pub mod geometry;
pub mod model;
pub mod physical_descriptions;
pub mod protocol;
pub mod revision;
pub mod values;
pub mod wheel;

/// Description of fixtures in a GDTF file.
///
/// Normally a `Description` is created while loading a GDTF file with
/// [`GdtfFile::new`](crate::GdtfFile::new). However a few alternatives are provided:
///  - Parsing a description XML manually with [`Self::from_reader`] or [`Self::from_str`].
///  - Constructing an empty description with [`Self::new`].
///
/// Corresponds to the root-level `<GDTF>` node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "GDTF")]
pub struct Description {
    /// Defines the minimal version of compatability.
    #[serde(rename = "@DataVersion")]
    pub data_version: Version,

    /// All fixture types defined in this file.
    #[serde(rename = "FixtureType", skip_serializing_if = "Vec::is_empty", default)]
    pub fixture_types: Vec<FixtureType>,
}

impl Description {
    /// Creates an empty description.
    ///
    /// The description will default to the latest GDTF version, currently `1.2`.
    pub fn new() -> Self {
        Description {
            data_version: Version { major: 1, minor: 2 },
            fixture_types: Vec::new(),
        }
    }

    /// Deserializes a GDTF description from a stream of XML data.
    ///
    /// If you already have a string use [`Self::from_str`]. If you have a `&[u8]` which is known to
    /// represent UTF-8, you can decode it first before using [`from_str`](Self::from_str).
    pub fn from_reader<R>(reader: R) -> GdtfResult<Self>
    where
        R: BufRead,
    {
        let mut de = quick_xml::de::Deserializer::from_reader(reader);
        Ok(serde_path_to_error::deserialize(&mut de)?)
    }

    /// Serializes as XML data into a `Write`r.
    pub fn to_writer<W>(&self, writer: W) -> GdtfResult<()>
    where
        W: Write,
    {
        Ok(quick_xml::se::to_writer(writer, self)?)
    }

    /// Serializes as XML data into a `String`.
    pub fn to_string(&self) -> GdtfResult<String> {
        Ok(quick_xml::se::to_string(self)?)
    }

    /// Looks up a [FixtureType] by [name](FixtureType::name).
    pub fn fixture_type(&self, name: &str) -> Option<&FixtureType> {
        self.fixture_types
            .iter()
            .find(|ty| ty.name.as_ref().map(Name::as_ref) == Some(name))
    }

    /// Performs validation checks on the object.
    pub fn validate<R: std::io::Read + 'static>(
        &self,
        resource_map: &mut ResourceMap<R>,
        result: &mut ValidationResult,
    ) {
        if !self.data_version.is_supported() {
            result.errors.push(ValidationError::new(
                ValidationObject::Description,
                None,
                ValidationErrorType::InvalidVersion(self.data_version),
            ));
        }

        let duplicate_fixture_type_name = self
            .fixture_types
            .iter()
            .filter_map(|fixture_type| fixture_type.name.as_ref())
            .find_duplicate();
        if let Some(name) = duplicate_fixture_type_name {
            result.errors.push(ValidationError::new(
                ValidationObject::FixtureType,
                name.to_string(),
                ValidationErrorType::DuplicateName,
            ));
        }

        for fixture_type in &self.fixture_types {
            fixture_type.validate(resource_map, result);
        }
    }
}

impl Default for Description {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for Description {
    type Err = GdtfError;

    /// Deserializes a GDTF description from an XML string.
    fn from_str(s: &str) -> GdtfResult<Self> {
        let mut de = quick_xml::de::Deserializer::from_str(s);
        Ok(serde_path_to_error::deserialize(&mut de)?)
    }
}
