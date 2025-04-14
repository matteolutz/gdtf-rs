//! The starting point of the description of a fixture type.

use crate::attribute::AttributeDefinitions;
use crate::description::util::IterUtil;
use crate::dmx_mode::DmxMode;
use crate::ft_preset::FtPreset;
use crate::geometry::{AnyGeometry, Geometry};
use crate::model::Model;
use crate::physical_descriptions::PhysicalDescriptions;
use crate::protocol::Protocols;
use crate::revision::Revision;
use crate::validation::{ValidationError, ValidationErrorType, ValidationObject, ValidationResult};
use crate::values::{non_empty_string, ok_or_default, Name};
use crate::wheel::Wheel;
use crate::{FtThumbnailFormat, ResourceMap};
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use uuid::Uuid;

/// The starting point of a description of a fixture type.
///
/// Corresponds to a `<FixtureType>` XML node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureType {
    /// Name of the fixture type used to be displayed as the file name and the library name.
    ///
    /// Corresponds to the `Name` XML attribute.
    #[serde(rename = "@Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<Name>,

    /// Short name of the fixture type.
    ///
    /// The short name should be as short as possible, but precise enough to describe the fixture
    /// type.
    ///
    /// Corresponds to the `ShortName` XML attribute.
    #[serde(rename = "@ShortName")]
    pub short_name: String,

    /// Detailed, complete name of the fixture type.
    ///
    /// Corresponds to the `LongName` XML attribute.
    #[serde(rename = "@LongName")]
    pub long_name: String,

    /// Manufacturer of the fixture type.
    ///
    /// Corresponds to the `Manufacturer` XML attribute.
    #[serde(rename = "@Manufacturer")]
    pub manufacturer: String,

    /// Description of the fixture type to be displayed in the library.
    ///
    /// Corresponds to the `Description` XML attribute.
    #[serde(rename = "@Description")]
    pub description: String,

    /// Unique number of the fixture type.
    ///
    /// Corresponds to the `FixtureTypeID` XML attribute.
    #[serde(rename = "@FixtureTypeID")]
    pub fixture_type_id: Uuid,

    /// File name without extension containing description of the thumbnail.
    ///
    /// One of the following files will be located in the root directory of the GDTF file:
    ///  - png file to provide a rasterized picture. Maximum resolution 1024x1024.
    ///  - svg file to provide a vector graphic.
    ///
    /// Read thumbnail resources with
    /// [ResourceMap::read_ft_thumbnail](crate::ResourceMap::read_ft_thumbnail).
    ///
    /// Corresponds to the `Thumbnail` XML attribute.
    #[serde(
        rename = "@Thumbnail",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_empty_string",
        default
    )]
    pub thumbnail: Option<String>,

    /// Horizontal offset in pixels from the top left of the viewbox to the insertion point on a
    /// label.
    ///
    /// Corresponds to the `ThumbnailOffsetX` XML attribute.
    #[serde(
        rename = "@ThumbnailOffsetX",
        deserialize_with = "ok_or_default",
        default
    )]
    pub thumbnail_offset_x: i32,

    /// Vertical offset in pixels from the top left of the viewbox to the insertion point on a
    /// label.
    ///
    /// Corresponds to the `ThumbnailOffsetY` XML attribute.
    #[serde(
        rename = "@ThumbnailOffsetY",
        deserialize_with = "ok_or_default",
        default
    )]
    pub thumbnail_offset_y: i32,

    /// Unique number (GUID) of a referenced fixture type.
    ///
    /// Corresponds to the `RefTF` XML attribute.
    #[serde(rename = "@RefTF", skip_serializing_if = "Option::is_none")]
    pub ref_tf: Option<Uuid>,

    /// Describes if it is possible to mount other devices to this device.
    ///
    /// Corresponds to the `CanHaveChildren` XML attribute.
    #[serde(
        rename = "@CanHaveChildren",
        serialize_with = "serialize_fixture_type_can_have_children",
        deserialize_with = "deserialize_fixture_type_can_have_children",
        default = "default_fixture_type_can_have_children"
    )]
    pub can_have_children: bool,

    /// Defines all Fixture Type Attributes that are used in the fixture type.
    ///
    /// Corresponds to the `<AttributeDefinitions>` XML child node.
    #[serde(rename = "AttributeDefinitions")]
    pub attribute_definitions: AttributeDefinitions,

    /// Defines the physical or virtual color wheels, gobo wheels, media server content and others.
    ///
    /// Corresponds to the `<Wheels>` XML child node.
    #[serde(
        rename = "Wheels",
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_wheels",
        deserialize_with = "deserialize_wheels",
        default
    )]
    pub wheels: Vec<Wheel>,

    /// Contains additional physical descriptions.
    ///
    /// Corresponds to the `<PhysicalDescriptions>` XML child node.
    #[serde(rename = "PhysicalDescriptions", default)]
    pub physical_descriptions: PhysicalDescriptions,

    /// Contains models of physically separated parts of the device.
    ///
    /// Corresponds to the `<Models>` XML child node.
    #[serde(
        rename = "Models",
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_models",
        deserialize_with = "deserialize_models",
        default
    )]
    pub models: Vec<Model>,

    /// Describes physically separated parts of the device.
    ///
    /// Corresponds to the `<Geometries>` XML child node.
    #[serde(
        rename = "Geometries",
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_geometries",
        deserialize_with = "deserialize_geometries"
    )]
    pub geometries: Vec<Geometry>,

    /// Contains descriptions of the DMX modes.
    ///
    /// Corresponds to the `<DMXModes>` XML child node.
    #[serde(
        rename = "DMXModes",
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_dmx_modes",
        deserialize_with = "deserialize_dmx_modes"
    )]
    pub dmx_modes: Vec<DmxMode>,

    /// Describes the history of the fixture type.
    ///
    /// Corresponds to the `<Revisions>` XML child node.
    #[serde(
        rename = "Revisions",
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_revisions",
        deserialize_with = "deserialize_revisions",
        default
    )]
    pub revisions: Vec<Revision>,

    /// Describes how to transfer user-defined and fixture type specific presets to other show files.
    ///
    /// Corresponds to the `<FTPresets>` XML child node.
    #[serde(
        rename = "FTPresets",
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_ft_presets",
        deserialize_with = "deserialize_ft_presets",
        default
    )]
    pub ft_presets: Vec<FtPreset>,

    /// Specified supported protocols.
    ///
    /// Corresponds to the `<Protocols>` XML child node.
    #[serde(rename = "Protocols", default)]
    pub protocols: Protocols,
}

impl FixtureType {
    /// Looks up a [Wheel] by [name](Wheel::name).
    pub fn wheel(&self, name: &str) -> Option<&Wheel> {
        self.wheels
            .iter()
            .find(|wheel| wheel.name.as_ref().map(Name::as_ref) == Some(name))
    }

    /// Looks up a [Model] by [name](Model::name).
    pub fn model(&self, name: &str) -> Option<&Model> {
        self.models
            .iter()
            .find(|model| model.name.as_ref().map(Name::as_ref) == Some(name))
    }

    /// Looks up a root-level [Geometry] by [name](Geometry::name).
    ///
    /// This will only return non-nested geometries. To search through all geometries, see
    /// [nested_geometry](FixtureType::nested_geometry). To find a specific geometry by
    /// [Node](super::values::Node)(super::values::Node) path,
    /// see [geometry_node](FixtureType::geometry_node).
    pub fn root_geometry(&self, name: &str) -> Option<&Geometry> {
        self.geometries
            .iter()
            .find(|geometry| geometry.name().map(Name::as_ref) == Some(name))
    }

    /// Looks up a nested [Geometry] by [name](Geometry::name).
    ///
    /// This will return any matching geometry in the tree, both root-level and nested. To search
    /// through only root-level geometries, see [root_geometry](FixtureType::root_geometry). To find
    /// a specific geometry by [Node](super::values::Node) path, see [geometry_node](FixtureType::geometry_node).
    pub fn nested_geometry(&self, name: &str) -> Option<&Geometry> {
        for geometry in &self.geometries {
            if geometry.name().map(Name::as_ref) == Some(name) {
                return Some(geometry);
            };
            if let Some(nested_child) = geometry.nested_child(name) {
                return Some(nested_child);
            }
        }
        None
    }

    /// Looks up a [Geometry] by [name](Geometry::name) from a path.
    ///
    /// This will return the geometry at the path indicated by the list of names. For example the
    /// path ["Base", "Head", "Lens"] will return the "Lens" geometry inside the "Head" geometry
    /// inside the "Base" geometry. To search through only root-level geometries, see
    /// [root_geometry](FixtureType::root_geometry). To search through all geometries, see
    /// [nested_geometry](FixtureType::nested_geometry).
    pub fn geometry_node(&self, node: &[Name]) -> Option<&Geometry> {
        let (root_name, tail) = node.split_first()?;
        self.root_geometry(root_name)?.child_node(tail)
    }

    /// Looks up a [DmxMode] by [name](DmxMode::name).
    pub fn dmx_mode(&self, name: &str) -> Option<&DmxMode> {
        self.dmx_modes
            .iter()
            .find(|dmx_mode| dmx_mode.name.as_ref().map(Name::as_ref) == Some(name))
    }

    /// Performs validation checks on the object.
    pub fn validate<R: std::io::Read + 'static>(
        &self,
        resource_map: &mut ResourceMap<R>,
        result: &mut ValidationResult,
    ) {
        if self.name.is_none() {
            result.errors.push(ValidationError::new(
                ValidationObject::FixtureType,
                None,
                ValidationErrorType::MissingName,
            ));
        }

        let name = self.name.as_ref();

        if let Some(thumbnail) = &self.thumbnail {
            let thumbnail_exists = resource_map
                .read_ft_thumbnail(thumbnail, FtThumbnailFormat::Svg)
                .is_ok()
                || resource_map
                    .read_ft_thumbnail(thumbnail, FtThumbnailFormat::Png)
                    .is_ok();
            if !thumbnail_exists {
                result.errors.push(ValidationError::new(
                    ValidationObject::FixtureType,
                    name.map(Name::to_string),
                    ValidationErrorType::ThumbnailNotFound(thumbnail.clone()),
                ));
            }
        }

        let duplicate_wheel_name = self
            .wheels
            .iter()
            .filter_map(|wheel| wheel.name.as_ref())
            .find_duplicate();
        if let Some(name) = duplicate_wheel_name {
            result.errors.push(ValidationError::new(
                ValidationObject::Wheel,
                name.to_string(),
                ValidationErrorType::DuplicateName,
            ));
        }

        let duplicate_model_name = self
            .models
            .iter()
            .filter_map(|model| model.name.as_ref())
            .find_duplicate();
        if let Some(name) = duplicate_model_name {
            result.errors.push(ValidationError::new(
                ValidationObject::Model,
                name.to_string(),
                ValidationErrorType::DuplicateName,
            ));
        }

        let duplicate_geometry_name = iter_geometry_recursive(&self.geometries)
            .filter_map(|geom| geom.name())
            .find_duplicate();
        if let Some(name) = duplicate_geometry_name {
            result.errors.push(ValidationError::new(
                ValidationObject::Geometry,
                name.to_string(),
                ValidationErrorType::DuplicateName,
            ));
        }

        let duplicate_dmx_mode_name = self
            .dmx_modes
            .iter()
            .filter_map(|dmx_mode| dmx_mode.name.as_ref())
            .find_duplicate();
        if let Some(name) = duplicate_dmx_mode_name {
            result.errors.push(ValidationError::new(
                ValidationObject::DmxMode,
                name.to_string(),
                ValidationErrorType::DuplicateName,
            ));
        }

        self.attribute_definitions.validate(result);

        for wheel in &self.wheels {
            wheel.validate(self, resource_map, result);
        }

        self.physical_descriptions.validate(result);

        for model in &self.models {
            model.validate(resource_map, result);
        }

        for geometry in &self.geometries {
            geometry.validate(self, result);
        }

        for dmx_mode in &self.dmx_modes {
            dmx_mode.validate(self, result);
        }

        self.protocols.validate(self, result);
    }
}

fn iter_geometry_recursive(geometry: &[Geometry]) -> Box<dyn Iterator<Item = &Geometry> + '_> {
    Box::new(
        geometry
            .iter()
            .flat_map(|geom| std::iter::once(geom).chain(iter_geometry_recursive(geom.children()))),
    )
}

fn default_fixture_type_can_have_children() -> bool {
    true
}

fn serialize_fixture_type_can_have_children<S>(
    value: &bool,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(match value {
        true => "Yes",
        false => "No",
    })
}

fn deserialize_fixture_type_can_have_children<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct EnumVisitor;

    impl<'de> Visitor<'de> for EnumVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("`Yes` or `No`")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v {
                "Yes" => Ok(true),
                "No" => Ok(false),
                _ => Err(E::invalid_value(Unexpected::Str(v), &self)),
            }
        }
    }

    deserializer.deserialize_str(EnumVisitor)
}

define_collect_helper!("Wheel" (serialize_wheels, deserialize_wheels) -> Wheel);
define_collect_helper!("Model" (serialize_models, deserialize_models) -> Model);
define_collect_helper!("$value" (serialize_geometries, deserialize_geometries) -> Geometry);
define_collect_helper!("DMXMode" (serialize_dmx_modes, deserialize_dmx_modes) -> DmxMode);
define_collect_helper!("Revision" (serialize_revisions, deserialize_revisions) -> Revision);
define_collect_helper!("FTPreset" (serialize_ft_presets, deserialize_ft_presets) -> FtPreset);
