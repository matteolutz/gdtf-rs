//! The General Device Type Format (GDTF) is an open standard for describing devices of the
//! entertainment industry. The latest version, 1.2, is standardised as
//! [DIN SPEC 15800:2022](https://www.beuth.de/en/technical-rule/din-spec-15800/349717520).
//!
//! This crate provides tools to read and inspect GDTF files. This is made up of three parts:
//!  - An object model which closely matches the structure defined in the GDTF specification.
//!  - A fairly lax parser capable of parsing mostly well-formed GDTF files into the object model.
//!  - A small number of utilities for validating and inspecting the object model.
//!
//! Importantly, the crate aims to stay close to the GDTF specification. It is not a goal to
//! provide a higher-level interface for fixtures represented by a GDTF file.
//!
//! # Example
//! ```
//! use gdtf_rs::GdtfFile;
//!
//! let file = std::fs::File::open("Generic@RGBW8@test.gdtf").expect("failed to read file");
//! let gdtf = GdtfFile::new(file).expect("failed to parse gdtf");
//!
//! println!("GDTF file defines {} fixture types", gdtf.description.fixture_types.len());
//! ```

#![deny(missing_docs, missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

use std::io::{BufReader, Read, Seek};
use zip::ZipArchive;

mod description;
mod resource;
mod result;
mod validation;

pub use self::description::*;
pub use self::resource::*;
pub use self::result::*;
pub use self::validation::*;

/// A GDTF file, the description and resources it provides.
///
/// GDTF files are zip files with a predefined structure. Each GDTF file contains:
///  - A `description.xml` file describing all relevant fixture types.
///  - Thumbnails for each fixture type.
///  - Icons for wheel slots in each fixture type.
///  - 3D models for the individual parts of each fixture type.
///
/// This library parses the description file and makes it available through the associated
/// [Description] value. Resources can be read through the associated [ResourceMap], based on the
/// names provided in the description.
#[derive(Debug)]
pub struct GdtfFile<R: Read + Seek + 'static> {
    /// Describes the GDTF file and all fixture types in the file.
    ///
    /// This is parsed from the `description.xml` file in the GDTF file.
    pub description: Description,

    /// Provides access to reading resource files in the GDTF file.
    pub resources: ResourceMap<R>,
}

impl<R: Read + Seek + 'static> GdtfFile<R> {
    /// Reads a GDTF file.
    ///
    /// This will immediately parse the description in the file.
    ///
    /// On error a [GdtfError] will be produced.
    pub fn new(reader: R) -> GdtfResult<Self> {
        let mut archive = ZipArchive::new(reader)?;

        let description_file = archive.by_name("description.xml")?;

        let description = {
            let buffered = BufReader::new(description_file);
            let mut de = quick_xml::de::Deserializer::from_reader(buffered);
            serde_path_to_error::deserialize(&mut de)?
        };
        let resources = ResourceMap::new(archive);

        Ok(GdtfFile {
            description,
            resources,
        })
    }

    /// Performs validation checks on the file.
    ///
    /// Validation errors do not impact the integrity of the file as a whole, but they may cause
    /// some features to not function as expected.
    pub fn validate(&mut self) -> ValidationResult {
        let mut result = ValidationResult::new();
        self.description.validate(&mut self.resources, &mut result);
        result
    }
}
