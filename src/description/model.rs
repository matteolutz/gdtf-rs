//! Defines the type and dimensions of models.

use crate::validation::{ValidationError, ValidationErrorType, ValidationObject, ValidationResult};
use crate::values::{non_empty_string, Name};
use crate::{Model2View, Model3Detail, Model3Format, ResourceMap};
use serde::{Deserialize, Serialize};

/// Defines the type and dimensions of the model.
///
/// Each device is divided into smaller parts: body, yoke, head and so on. These are called
/// geometries. Each geometry has a separate model description and a physical description.
///
/// Corresponds to a `<Model>` XML node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Model {
    /// The unique name of the model.
    ///
    /// Corresponds to the `Name` XML attribute.
    #[serde(rename = "@Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<Name>,

    /// Length of the model in meters.
    ///
    /// Corresponds to the `Length` XML attribute.
    #[serde(rename = "@Length", default)]
    pub length: f64,

    /// Width of the model in meters.
    ///
    /// Corresponds to the `Width` XML attribute.
    #[serde(rename = "@Width", default)]
    pub width: f64,

    /// Height of the model in meters.
    ///
    /// Corresponds to the `Height` XML attribute.
    #[serde(rename = "@Height", default)]
    pub height: f64,

    /// Type of 3D model.
    ///
    /// Corresponds to the `PrimitiveType` XML attribute.
    #[serde(rename = "@PrimitiveType", default)]
    pub primitive_type: PrimitiveType,

    /// Optional file name without extension and without subfolder containing description of the
    /// model.
    ///
    /// Provided as either:
    ///  - A 3DS or GLB file to provide a 3D model.
    ///  - An SVG file to provide a 2D symbol.
    ///
    /// Several files with the same name and different formats may be present. The resource files
    /// are located in subfolders of the `./models` folder. The names of the subfolders correspond
    /// to the file format of the resource files (3ds, gltf, svg). Additionaly two extra levels
    /// of detail may be provided for 3D models in the folders 3ds_low, gltf_low, 3ds_high and
    /// gltf_high.
    ///
    /// All models of a device combined should not exceed a maximum vertex count of 1200 for the
    /// default mesh level of detail.
    ///
    /// Read model resources with
    /// [ResourceMap::read_model_symbol](crate::ResourceMap::read_model_symbol) for 2D symbols, and
    /// [ResourceMap::read_model_mesh](crate::ResourceMap::read_model_mesh) for 3D meshes.
    ///
    /// Corresponds to the `File` XML attribute.
    #[serde(
        rename = "@File",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_empty_string",
        default
    )]
    pub file: Option<String>,

    /// Offset on the X axis from 0,0 to the desired insertion point of the top view SVG.
    ///
    /// Corresponds to the `SVGOffsetX` XML attribute.
    #[serde(rename = "@SVGOffsetX", default)]
    pub svg_offset_x: f64,

    /// Offset on the Y axis from 0,0 to the desired insertion point of the top view SVG.
    ///
    /// Corresponds to the `SVGOffsetY` XML attribute.
    #[serde(rename = "@SVGOffsetY", default)]
    pub svg_offset_y: f64,

    /// Offset on the X axis from 0,0 to the desired insertion point of the side view SVG.
    ///
    /// Corresponds to the `SVGSideOffsetX` XML attribute.
    #[serde(rename = "@SVGSideOffsetX", default)]
    pub svg_side_offset_x: f64,

    /// Offset on the Y axis from 0,0 to the desired insertion point of the side view SVG.
    ///
    /// Corresponds to the `SVGSideOffsetY` XML attribute.
    #[serde(rename = "@SVGSideOffsetY", default)]
    pub svg_side_offset_y: f64,

    /// Offset on the X axis from 0,0 to the desired insertion point of the front view SVG.
    ///
    /// Corresponds to the `SVGFrontOffsetX` XML attribute.
    #[serde(rename = "@SVGFrontOffsetX", default)]
    pub svg_front_offset_x: f64,

    /// Offset on the Y axis from 0,0 to the desired insertion point of the front view SVG.
    ///
    /// Corresponds to the `SVGFrontOffsetY` XML attribute.
    #[serde(rename = "@SVGFrontOffsetY", default)]
    pub svg_front_offset_y: f64,
}

impl Model {
    /// Performs validation checks on the object.
    pub fn validate<R: std::io::Read + 'static>(
        &self,
        resource_map: &mut ResourceMap<R>,
        result: &mut ValidationResult,
    ) {
        if self.name.is_none() {
            result.errors.push(ValidationError::new(
                ValidationObject::Model,
                None,
                ValidationErrorType::MissingName,
            ));
        }

        let name = self.name.as_ref();

        if let Some(file) = &self.file {
            let model_exists = resource_map
                .read_model_symbol(file, Model2View::Front)
                .is_ok()
                || resource_map
                    .read_model_mesh(file, Model3Format::Gltf, Model3Detail::Default)
                    .is_ok()
                || resource_map
                    .read_model_mesh(file, Model3Format::Max3ds, Model3Detail::Default)
                    .is_ok();
            if !model_exists {
                result.errors.push(ValidationError::new(
                    ValidationObject::Model,
                    name.map(Name::to_string),
                    ValidationErrorType::MediaNotFound(file.clone()),
                ));
            }
        }
    }
}

/// Type of 3D [Model].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PrimitiveType {
    /// Cube primitive. Not defined in specification.
    Cube,

    /// Cylinder primitive. Not defined in specification.
    Cylinder,

    /// Sphere primitive. Not defined in specification.
    Sphere,

    /// Base primitive. Not defined in specification.
    Base,

    /// Yoke primitive. Not defined in specification.
    Yoke,

    /// Head primitive. Not defined in specification.
    Head,

    /// Scanner primitive. Not defined in specification.
    Scanner,

    /// Conventional primitive. Not defined in specification.
    Conventional,

    /// Pigtail primitive. Not defined in specification.
    Pigtail,

    /// Base1_1 primitive. Not defined in specification.
    Base1_1,

    /// Scanner1_1 primitive. Not defined in specification.
    Scanner1_1,

    /// Conventional1_1 primitive. Not defined in specification.
    Conventional1_1,

    /// Undefined primitive. Not defined in specification.
    #[default]
    #[serde(other)]
    Undefined,
}
