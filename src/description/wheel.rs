//! Properties of all physical or virtual wheels of a device.
//!
//! Note 1: Physical or virtual wheels represent the changes to the light beam within the device.
//! Typically color, gobo, prism, animation, content and others are described by wheels.

use crate::description::util::IterUtil;
use crate::fixture_type::FixtureType;
use crate::physical_descriptions::Filter;
use crate::validation::{ValidationError, ValidationErrorType, ValidationObject, ValidationResult};
use crate::values::{non_empty_string, ColorCie, Name, Node, NodeExt, Rotation};
use crate::ResourceMap;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

/// Defines all physical or virtual wheels of the device.
///
/// Corresponds to a `<Wheel>` XML node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wheel {
    /// The unique name of the wheel.
    ///
    /// Corresponds to the `Name` XML attribute.
    #[serde(rename = "@Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<Name>,

    /// A list of all slots on the wheel.
    ///
    /// Corresponds to all `<Slot>` XML child nodes.
    #[serde(rename = "Slot", skip_serializing_if = "Vec::is_empty", default)]
    pub slots: Vec<WheelSlot>,
}

impl Wheel {
    /// Looks up a [WheelSlot] by [name](WheelSlot::name).
    pub fn slot(&self, name: &str) -> Option<&WheelSlot> {
        self.slots
            .iter()
            .find(|slot| slot.name.as_ref().map(Name::as_ref) == Some(name))
    }

    /// Performs validation checks on the object.
    pub fn validate<R: std::io::Read + 'static>(
        &self,
        parent_fixture_type: &FixtureType,
        resource_map: &mut ResourceMap<R>,
        result: &mut ValidationResult,
    ) {
        if self.name.is_none() {
            result.errors.push(ValidationError::new(
                ValidationObject::Wheel,
                None,
                ValidationErrorType::MissingName,
            ));
        }

        let duplicate_wheel_slot_name = self
            .slots
            .iter()
            .filter_map(|slot| slot.name.as_ref())
            .find_duplicate();
        if let Some(name) = duplicate_wheel_slot_name {
            result.errors.push(ValidationError::new(
                ValidationObject::WheelSlot,
                name.to_string(),
                ValidationErrorType::DuplicateName,
            ));
        }

        for slot in &self.slots {
            slot.validate(parent_fixture_type, resource_map, result);
        }
    }
}

/// Represents a slot on a [Wheel].
///
/// The link between a lot and a [ChannelSet](super::dmx_mode::ChannelSet) is done via the wheel
/// slot index. The wheel slot index of a slot is derived from the order of a wheel's slots. The
/// wheel slot index is normalized to 1.
///
/// Corresponds to a `<Slot>` XML node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WheelSlot {
    /// The unique name of the wheel slot.
    ///
    /// Corresponds to the `Name` XML attribute.
    #[serde(rename = "@Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<Name>,

    /// Optics of the slot, either a color or a filter.
    ///
    /// Corresponds to the `Color` or `Filter` XML attribute.
    #[serde(flatten, default)]
    pub optic: WheelSlotOptic,

    /// An optional PNG file name without extension containing an image for specific gobos etc.
    ///
    ///  - Maximum resolution of picture: 1024x1024
    ///  - Recommended resolution of gobo: 256x256
    ///  - Recommended resolution of animation wheel: 256x256
    ///
    /// These resource files are located in a folder called `./wheels` in the GDTF archive.
    ///
    /// Read wheel slot media resources with
    /// [ResourceMap::read_wheel_media](crate::ResourceMap::read_wheel_media).
    ///
    /// Note 1: More information on the definitions of images used in wheel slots to visualize
    /// gobos, animation wheels or color wheels can be found in
    /// [Annex E](https://gdtf.eu/gdtf/annex/annex-e/) of the GDTF specification.
    ///
    /// Corresponds to the `MediaFileName` XML attribute.
    #[serde(
        rename = "@MediaFileName",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "non_empty_string",
        default
    )]
    pub media_name: Option<String>,

    /// If the wheel slot has a prism, a list of prism facets. Otherwise empty.
    ///
    /// Corresponds to all `<Facet>` XML child nodes.
    #[serde(rename = "Facet", skip_serializing_if = "Vec::is_empty", default)]
    pub facets: Vec<PrismFacet>,

    /// If the wheel slot has an animation wheel, defines the optics of the animation.
    ///
    /// Corresponds to the `<AnimationSystem>` XML child node.
    #[serde(rename = "AnimationSystem", skip_serializing_if = "Option::is_none")]
    pub animation_system: Option<AnimationSystem>,
}

impl WheelSlot {
    /// Looks up the optional [Filter] linked by this wheel slot.
    pub fn filter<'s>(&self, parent_fixture_type: &'s FixtureType) -> Option<&'s Filter> {
        let WheelSlotOptic::Filter(node) = &self.optic else {
            return None;
        };
        let filter_name = node.single()?;
        parent_fixture_type
            .physical_descriptions
            .filter(filter_name)
    }

    /// Performs validation checks on the object.
    pub fn validate<R: std::io::Read + 'static>(
        &self,
        parent_fixture_type: &FixtureType,
        resource_map: &mut ResourceMap<R>,
        result: &mut ValidationResult,
    ) {
        if self.name.is_none() {
            result.errors.push(ValidationError::new(
                ValidationObject::WheelSlot,
                None,
                ValidationErrorType::MissingName,
            ));
        }

        let name = self.name.as_ref();

        if let Some(media_name) = &self.media_name {
            let media_exists = resource_map.read_wheel_media(media_name).is_ok();
            if !media_exists {
                result.errors.push(ValidationError::new(
                    ValidationObject::WheelSlot,
                    name.map(Name::to_string),
                    ValidationErrorType::MediaNotFound(media_name.clone()),
                ));
            }
        }

        if let (WheelSlotOptic::Filter(node), None) =
            (&self.optic, self.filter(parent_fixture_type))
        {
            result.errors.push(ValidationError::new(
                ValidationObject::WheelSlot,
                name.map(Name::to_string),
                ValidationErrorType::LinkNotFound(ValidationObject::Filter, node.clone()),
            ));
        }
    }
}

/// Defines the optics of a [WheelSlot].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WheelSlotOptic {
    /// Color of the wheel slot.
    ///
    /// Y is relative to the overall output defined in the luminous flux of the related
    /// [BeamGeometry](super::geometry::BeamGeometry) (transmissive case).
    ///
    /// Corresponds to the `Color` XML attribute.
    #[serde(rename = "@Color")]
    Color(ColorCie),

    /// Link to a [Filter] on the wheel slot.
    ///
    /// The link is relative to the parent [FixtureType] object.
    ///
    /// See the [WheelSlot::filter] method to look up the linked filter.
    ///
    /// Corresponds to the `Filter` XML attribute.
    #[serde(rename = "@Filter")]
    Filter(Node),
}

impl Default for WheelSlotOptic {
    fn default() -> Self {
        WheelSlotOptic::Color(ColorCie::white())
    }
}

/// Defines a prism facet on the prism wheel slot.
///
/// Corresponds to a `<Facet>` XML node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrismFacet {
    /// Color of the prism facet.
    ///
    /// Corresponds to the `Color` XML attribute.
    #[serde(rename = "@Color", default = "ColorCie::white")]
    pub color: ColorCie,

    /// The rotation, translation and scaling of the facet.
    ///
    /// Corresponds to the `Rotation` XML attribute.
    #[serde(rename = "@Rotation")]
    pub rotation: Rotation,
}

/// Defines the animation system disk on the animation wheel slot.
///
/// The system is defined by a three point spline, which describes the path of the animation system
/// in the beam in relation to the middle of the media file.
///
/// Corresponds to an `<AnimationSystem>` XML node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationSystem {
    /// First point of the spline as an X and Y value.
    ///
    /// Corresponds to the `P1` XML attribute.
    #[serde(
        rename = "@P1",
        serialize_with = "serialize_xy_array",
        deserialize_with = "deserialize_xy_array"
    )]
    pub p1: [f64; 2],

    /// Second point of the spline as an X and Y value.
    ///
    /// Corresponds to the `P2` XML attribute.
    #[serde(
        rename = "@P2",
        serialize_with = "serialize_xy_array",
        deserialize_with = "deserialize_xy_array"
    )]
    pub p2: [f64; 2],

    /// Third point of the spline as an X and Y value.
    ///
    /// Corresponds to the `P3` XML attribute.
    #[serde(
        rename = "@P3",
        serialize_with = "serialize_xy_array",
        deserialize_with = "deserialize_xy_array"
    )]
    pub p3: [f64; 2],

    /// Radius of the circle that defines the section of the media file which will be shown in the
    /// beam.
    ///
    /// Corresponds to the `Radius` XML attribute.
    #[serde(rename = "@Radius")]
    pub radius: f64,
}

fn serialize_xy_array<S>(value: &[f64; 2], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{},{}", value[0], value[1]))
}

fn deserialize_xy_array<'de, D>(deserializer: D) -> Result<[f64; 2], D::Error>
where
    D: Deserializer<'de>,
{
    struct XyArrayVisitor;

    impl<'de> Visitor<'de> for XyArrayVisitor {
        type Value = [f64; 2];

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a floating point array in the format float,float")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let values = v.split(',').map(|value_str| {
                value_str.trim().parse::<f64>().map_err(|_| {
                    E::invalid_value(Unexpected::Str(value_str), &"a floating point number")
                })
            });
            take_two(values, || E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_str(XyArrayVisitor)
}

fn take_two<I: Iterator<Item = Result<T, E>>, T, E>(
    mut iter: I,
    on_missing: impl FnOnce() -> E,
) -> Result<[T; 2], E> {
    let val0 = match iter.next() {
        Some(val0) => val0?,
        None => return Err(on_missing()),
    };
    let val1 = match iter.next() {
        Some(val1) => val1?,
        None => return Err(on_missing()),
    };
    if iter.next().is_some() {
        Err(on_missing())
    } else {
        Ok([val0, val1])
    }
}
