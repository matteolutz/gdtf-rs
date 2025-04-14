use crate::{GdtfError, GdtfResult};
use std::fmt::{Debug, Formatter};
use std::io::{Read, Seek};
use zip::read::ZipFile;
use zip::result::{ZipError, ZipResult};
use zip::ZipArchive;

/// Provides resource files contained in a GDTF file.
///
/// GDTF files contain various kinds of resources:
///  - Thumbnails for each fixture type.
///  - Icons for wheel slots in each fixture type.
///  - 3D models for the individual parts of each fixture type.
///
/// Each resource is accessed by a name, which is usually provided by the GDTF file description.
/// Most resources can be provided in several formats, or with several alternatives. The desired
/// format and alternative is specified when accessing the resource.
pub struct ResourceMap<R: Read> {
    archive: Box<dyn AnyZipArchive<R>>,
}

impl<R: Read + 'static> ResourceMap<R> {
    pub(crate) fn new<A>(archive: A) -> Self
    where
        A: AnyZipArchive<R>,
    {
        ResourceMap {
            archive: Box::new(archive),
        }
    }

    /// Opens a resource file contained in the GDTF file for reading.
    pub fn read_resource(&mut self, path: &str) -> GdtfResult<Resource<R>> {
        match self.archive.by_name(path) {
            Ok(file) => Ok(Resource::new(file)),
            Err(ZipError::FileNotFound) => Err(GdtfError::ResourceNotFound),
            Err(err) => Err(err.into()),
        }
    }

    /// Opens a fixture type thumbnail resource for reading.
    ///
    /// Thumbnail names are typically provided by the
    /// [FixtureType thumbnail](crate::fixture_type::FixtureType::thumbnail) property. The GDTF
    /// file can provide the thumbnail in two formats:
    ///
    ///  - PNG file to provide a rasterized picture. Maximum resolution 1024x1024.
    ///  - SVG file to provide a vector graphic.
    pub fn read_ft_thumbnail(
        &mut self,
        name: &str,
        format: FtThumbnailFormat,
    ) -> GdtfResult<Resource<R>> {
        self.read_resource(&format!("{name}.{}", format.extension()))
    }

    /// Opens a wheel media resource for reading.
    ///
    /// Wheel media names are typically provided by the
    /// [WheelSlot media_name](crate::wheel::WheelSlot::media_name) property. The GDTF file provides
    /// the resource as a PNG file.
    ///
    ///  - Maximum resolution of picture: 1024x1024
    ///  - Recommended resolution of gobo: 256x256
    ///  - Recommended resolution of animation wheel: 256x256
    pub fn read_wheel_media(&mut self, name: &str) -> GdtfResult<Resource<R>> {
        self.read_resource(&format!("wheels/{name}.png"))
    }

    /// Opens a 2D model file for reading.
    ///
    /// Model file names are typically provided by the [Model file](crate::model::Model::file)
    /// property. The GDTF file provides the resource in three formats:
    ///
    ///  - SVG files to provide a 2D symbol
    ///  - 3DS or GLTF files to provide a 3D mesh
    ///
    /// Separate SVG resources can be provided for three different view directions:
    ///  - Top view: view from top in -Z direction.
    ///  - Front view: view from front in Y direction.
    ///  - Side view: view from front in -X direction.
    ///
    /// The special SVG color `#C8C8C8` indicates the background of the symbol. Software may replace
    /// this color with another color.
    ///
    /// Any SVG files should follow these requirements:
    ///  - Use SVG 1.1 spec
    ///  - Don't embed bitmap images.
    ///  - Align the viewbox to the top left of the device.
    ///
    /// To read 3D (3DS or GLB) model files, see [read_model_mesh](Self::read_model_mesh).
    pub fn read_model_symbol(&mut self, name: &str, view: Model2View) -> GdtfResult<Resource<R>> {
        self.read_resource(&format!("models/{}/{name}.svg", view.folder()))
    }

    /// Opens a 3D model file for reading.
    ///
    /// Model file names are typically provided by the [Model file](crate::model::Model::file)
    /// property. The GDTF file provides the resource in three formats:
    ///
    ///  - SVG files to provide a 2D symbol
    ///  - 3DS or GLTF files to provide a 3D mesh
    ///
    /// Separate mesh resources can be provided for three different levels of detail:
    ///  - Low: optional, this is the mesh for fixtures that are far away from the camera. They
    ///    should have a vertex count roughly 30% of the default mesh.
    ///  - Default: this is the default mesh that is used for real time visualization in
    ///    preprogramming tools. It should have the minimum vertex count possible, while still
    ///    looking like the fixture in 3D.
    ///  - High: optional, this is the high quality mesh targeting non-realtime applications, where
    ///    the vertex count is not that important. There is no limit for the vertex count.
    ///
    /// Any GLTF (GLB) files should follow these requirements:
    ///  - Use the `glb` binary format.
    ///  - Only use the 2.0 version.
    ///  - Do not use extensions.
    ///  - Do not use animations.
    ///  - Only use jpeg or png texture resources.
    ///  - All vertex attributes are `GL_FLOAT`.
    ///
    /// To read 2D (SVG) model files, see [read_model_symbol](Self::read_model_symbol).
    pub fn read_model_mesh(
        &mut self,
        name: &str,
        format: Model3Format,
        detail: Model3Detail,
    ) -> GdtfResult<Resource<R>> {
        self.read_resource(&format!(
            "models/{}{}/{name}.{}",
            format.folder(),
            detail.folder_postfix(),
            format.extension()
        ))
    }
}

impl<R: Read> Debug for ResourceMap<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceMap").finish()
    }
}

pub(crate) trait AnyZipArchive<R: Read>: 'static {
    fn by_name(&mut self, path: &str) -> ZipResult<ZipFile<R>>;
}

impl<R> AnyZipArchive<R> for ZipArchive<R>
where
    R: Read + Seek + 'static,
{
    fn by_name(&mut self, path: &str) -> ZipResult<ZipFile<R>> {
        self.by_name(path)
    }
}

/// A resource contained in a GDTF file.
///
/// Resources contain binary data which is exposed as a stream through the [Read] trait. How to
/// interpret the data depends on the type of resource.
pub struct Resource<'a, R: Read> {
    archive_file: ZipFile<'a, R>,
}

impl<'a, R: Read> Resource<'a, R> {
    pub(crate) fn new(archive_file: ZipFile<'a, R>) -> Self {
        Resource { archive_file }
    }

    /// Returns the size of the resource file in bytes.
    pub fn size(&self) -> u64 {
        self.archive_file.size()
    }
}

impl<'a, R: Read> Debug for Resource<'a, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("archive_file", &self.archive_file.name())
            .finish()
    }
}

impl<'a, R: Read> Read for Resource<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.archive_file.read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.archive_file.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.archive_file.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.archive_file.read_exact(buf)
    }
}

/// Format to use when reading a thumbnail resource.
///
/// Thumbnail resources are read with [ResourceMap::read_ft_thumbnail].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FtThumbnailFormat {
    /// PNG image file.
    Png,

    /// SVG image file.
    Svg,
}

impl FtThumbnailFormat {
    /// Returns the extension of the format, without a `.`.
    pub const fn extension(self) -> &'static str {
        match self {
            FtThumbnailFormat::Png => "png",
            FtThumbnailFormat::Svg => "svg",
        }
    }
}

/// View direction used when reading a 2D model symbol.
///
/// 2D model symbols are read with [ResourceMap::read_model_symbol].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model2View {
    /// Symbol to display on top of the model.
    Top,

    /// Symbol to display on the front of the model.
    Front,

    /// Symbol to display on the sides of the model.
    Side,
}

impl Model2View {
    /// Returns the folder containing resource files for this view direction.
    pub const fn folder(self) -> &'static str {
        match self {
            Model2View::Top => "svg",
            Model2View::Front => "svg_front",
            Model2View::Side => "svg_side",
        }
    }
}

/// Format used when reading a 3D model mesh.
///
/// 3D model meshes are read with [ResourceMap::read_model_mesh].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model3Format {
    /// GLTF mesh file.
    Gltf,

    /// 3DS mesh file.
    Max3ds,
}

impl Model3Format {
    /// Returns the extension of the format, without a `.`.
    pub const fn extension(self) -> &'static str {
        match self {
            Model3Format::Gltf => "glb",
            Model3Format::Max3ds => "3ds",
        }
    }

    /// Returns the folder containing resource files in this format.
    pub const fn folder(self) -> &'static str {
        match self {
            Model3Format::Gltf => "gltf",
            Model3Format::Max3ds => "3ds",
        }
    }
}

/// Level of detail used when reading a 3D model mesh.
///
/// 3D model meshes are read with [ResourceMap::read_model_mesh].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model3Detail {
    /// Mesh for fixtures that are far away from the camera. They should have a vertex count roughly
    /// 30% of the default mesh.
    Low,

    /// Default mesh that is used for real time visualization in preprogramming tools. It should
    /// have the minimum vertex count possible, while still looking like the fixture in 3D.
    Default,

    /// High quality mesh targeting non-realtime applications, where the vertex count is not that
    /// important. There is no limit for the vertex count.
    High,
}

impl Model3Detail {
    /// Returns the folder containing resource files for this level of detail.
    pub const fn folder_postfix(self) -> &'static str {
        match self {
            Model3Detail::Low => "_low",
            Model3Detail::Default => "",
            Model3Detail::High => "_high",
        }
    }
}
