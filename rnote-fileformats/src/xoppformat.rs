use std::io::{Read, Write};

use roxmltree::{Node, NodeType};
use serde::{Deserialize, Serialize};

use super::{AsXmlAttributeValue, FileFormatLoader, FileFormatSaver, XmlLoadable, XmlWritable};

/// Compress bytes with gzip
fn compress_to_gzip(to_compress: &[u8], file_name: &str) -> Result<Vec<u8>, anyhow::Error> {
    let compressed_bytes = Vec::<u8>::new();

    let mut encoder = flate2::GzBuilder::new()
        .filename(file_name)
        .write(compressed_bytes, flate2::Compression::default());

    encoder.write_all(to_compress)?;

    Ok(encoder.finish()?)
}

/// Decompress from gzip
fn decompress_from_gzip(compressed: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut decoder = flate2::read::MultiGzDecoder::new(compressed);
    let mut bytes: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut bytes)?;

    Ok(bytes)
}

/// Represents a Xournal++ `.xopp` file.
/// The original Xournal spec can be found here: <http://xournal.sourceforge.net/manual.html#file-format>
/// The coordinates units saved to a .xopp are in 72dpi, meaning a vector of (1,0) has a length of 1 / 72 inch.
#[derive(Debug)]
pub struct XoppFile {
    /// The .xopp XML root element
    pub xopp_root: XoppRoot,
}

impl FileFormatLoader for XoppFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let decompressed = String::from_utf8(decompress_from_gzip(&bytes)?)?;

        let options = roxmltree::ParsingOptions::default();
        let parsed_doc = roxmltree::Document::parse_with_options(decompressed.as_str(), options)?;
        let mut xopp_root = XoppRoot::default();

        xopp_root.load_from_xml(parsed_doc.root_element())?;

        Ok(Self { xopp_root })
    }
}

impl FileFormatSaver for XoppFile {
    fn save_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let options = xmlwriter::Options::default();
        let mut xml_writer = xmlwriter::XmlWriter::new(options);
        self.xopp_root.write_to_xml(&mut xml_writer);
        let output = xml_writer.end_document();

        let compressed = compress_to_gzip(output.as_bytes(), file_name)?;

        Ok(compressed)
    }
}

impl XoppFile {
    /// The DPI of the Xopp file, is hardcoded to 72 DPI
    pub const DPI: f64 = 72.0;
}

/// Represents a Xournal++ XML root element
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct XoppRoot {
    /// The file version
    pub fileversion: String,
    /// The file title
    pub title: String,
    /// A preview image, encoded as base64
    pub preview: String,
    /// The pages elements
    pub pages: Vec<XoppPage>,
}

impl XmlLoadable for XoppRoot {
    fn load_from_xml(&mut self, root_node: Node) -> anyhow::Result<()> {
        if let Some(fileversion) = root_node.attribute("fileversion") {
            self.fileversion = fileversion.to_string();
        }

        for child in root_node.children() {
            match child.node_type() {
                NodeType::Element => match child.tag_name().name() {
                    "title" => {
                        if let Some(title) = child.text() {
                            self.title = title.to_string();
                        }
                    }
                    "preview" => {
                        if let Some(preview) = child.text() {
                            self.preview = preview.to_string();
                        }
                    }
                    "page" => {
                        let mut new_page = XoppPage::default();
                        new_page.load_from_xml(child)?;
                        self.pages.push(new_page);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }
}

impl XmlWritable for XoppRoot {
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter) {
        w.start_element("xournal");
        w.write_attribute("fileversion", &self.fileversion);
        w.start_element("title");
        w.write_text(&self.title);
        w.end_element();
        for page in self.pages.iter() {
            page.write_to_xml(w);
        }
        w.end_element();
    }
}

/// Represents a Xopp Page
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct XoppPage {
    /// The width of the page
    pub width: f64,
    /// The height of the page
    pub height: f64,
    /// The Background of the page
    pub background: XoppBackground,
    /// The layers of the page
    pub layers: Vec<XoppLayer>,
}

impl XmlLoadable for XoppPage {
    fn load_from_xml(&mut self, node: Node) -> anyhow::Result<()> {
        self.width = node
            .attribute("width")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse width attribute of XoppPage for node with id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .parse::<f64>()?;

        self.height = node
            .attribute("height")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse height attribute of XoppPage with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .parse::<f64>()?;

        for child in node.children() {
            match child.node_type() {
                NodeType::Element => match child.tag_name().name() {
                    "background" => {
                        self.background.load_from_xml(child)?;
                    }
                    "layer" => {
                        let mut new_layer = XoppLayer::default();
                        new_layer.load_from_xml(child)?;
                        self.layers.push(new_layer);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(())
    }
}

impl XmlWritable for XoppPage {
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter) {
        w.start_element("page");
        w.write_attribute("width", &format!("{}", self.width));
        w.write_attribute("height", &format!("{}", self.height));
        self.background.write_to_xml(w);
        for layer in self.layers.iter() {
            layer.write_to_xml(w);
        }
        w.end_element()
    }
}

/// Represents a Xopp Background type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XoppBackgroundType {
    /// A solid background with a color and style
    Solid {
        /// The color
        color: XoppColor,
        /// The solid background style
        style: XoppBackgroundSolidStyle,
    },
    /// A background with a pixmap
    Pixmap {
        /// The domain for the pixmap
        domain: XoppBackgroundPixmapDomain,
        /// The filename that is to the image for the pixmap
        filename: String,
    },
    /// A background with a pdf ( NOT IMPLEMENTED )
    Pdf,
}

impl XmlWritable for XoppBackgroundType {
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter) {
        match self {
            Self::Solid { color, style } => {
                w.write_attribute("type", "solid");
                w.write_attribute("color", &color.as_xml_attr_value());
                w.write_attribute("style", &style.as_xml_attr_value());
            }
            Self::Pixmap { domain, filename } => {
                w.write_attribute("type", "pixmap");
                w.write_attribute("domain", &domain.as_xml_attr_value());
                w.write_attribute("filename", filename);
            }
            Self::Pdf => {
                w.write_attribute("type", "pdf");
            }
        }
    }
}

impl Default for XoppBackgroundType {
    fn default() -> Self {
        Self::Solid {
            color: XoppColor {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0xff,
            },
            style: XoppBackgroundSolidStyle::default(),
        }
    }
}

/// The xopp background style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XoppBackgroundSolidStyle {
    /// The plain background style
    Plain,
    /// The lined background style
    Lined,
    /// The ruled background style
    Ruled,
    /// The graph background style
    Graph,
}

impl Default for XoppBackgroundSolidStyle {
    fn default() -> Self {
        Self::Plain
    }
}

impl AsXmlAttributeValue for XoppBackgroundSolidStyle {
    fn as_xml_attr_value(&self) -> String {
        match self {
            Self::Plain => String::from("plain"),
            Self::Lined => String::from("lined"),
            Self::Ruled => String::from("ruled"),
            Self::Graph => String::from("graph"),
        }
    }
}

/// The Xopp background pixmap domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XoppBackgroundPixmapDomain {
    /// absolute
    Absolute,
    /// attach
    Attach,
    /// clone
    Clone,
}

impl AsXmlAttributeValue for XoppBackgroundPixmapDomain {
    fn as_xml_attr_value(&self) -> String {
        match self {
            Self::Absolute => String::from("absolute"),
            Self::Attach => String::from("attach"),
            Self::Clone => String::from("clone"),
        }
    }
}

impl Default for XoppBackgroundPixmapDomain {
    /// The default pipxmap domain
    fn default() -> Self {
        Self::Absolute
    }
}

/// The Xopp Background
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct XoppBackground {
    /// Optional background name
    pub name: Option<String>,
    /// The background type
    pub bg_type: XoppBackgroundType,
}

impl XmlLoadable for XoppBackground {
    fn load_from_xml(&mut self, node: Node) -> anyhow::Result<()> {
        self.name = node.attribute("name").map(|name| name.to_string());

        match node.attribute("type").ok_or_else(|| {
            anyhow::anyhow!(
                "failed to parse `type` attribute of XoppBackground with node id {:?}, could not find attribute",
                node.id()
            )
        })? {
            "solid" => {
                let style = match node.attribute("style").ok_or_else(|| {
                    anyhow::anyhow!("failed to parse `style` attribute in XoppBackground with node id {:?}, could not find attribute", node.id())
                })? {
                    "plain" => XoppBackgroundSolidStyle::Plain,
                    "lined" => XoppBackgroundSolidStyle::Lined,
                    "ruled" => XoppBackgroundSolidStyle::Ruled,
                    "graph" => XoppBackgroundSolidStyle::Graph,
                    _ => {
                        return Err(anyhow::anyhow!("Err while parsing `style` attribute of XoppBackground with node id {:?}, is not a valid value", node.id()));
                    }
                };

                let color = XoppColor::from_backgroundcolor_attr_value(
                    node.attribute("color").ok_or_else(|| {
                        anyhow::anyhow!(
                            "failed to parse `color` attribute in XoppBackground with id {:?}",
                            node.id()
                        )
                    })?,
                )?;
                self.bg_type = XoppBackgroundType::Solid { color, style };
            }
            "pixmap" => {
                let domain = match node.attribute("domain").ok_or_else(|| {
                    anyhow::anyhow!("failed to parse `domain` attribute in XoppBackground with node id {:?}, could not find attribute", node.id())
                })? {
                    "absolute" => XoppBackgroundPixmapDomain::Absolute,
                    "attach" => XoppBackgroundPixmapDomain::Attach,
                    "clone" => XoppBackgroundPixmapDomain::Clone,
                    _ => {
                        return Err(anyhow::anyhow!("Err while parsing `style` attribute of XoppBackground with node id {:?}, is not a valid value", node.id()));
                    }
                };
                let filename = node
                    .attribute("filename")
                    .ok_or_else(|| {
                        anyhow::anyhow!("failed to parse `filename` attribute in XoppBackground with node id {:?}, could not find attribute", node.id())
                    })?
                    .to_string();
                self.bg_type = XoppBackgroundType::Pixmap { domain, filename };
            }
            "pdf" => {
                self.bg_type = XoppBackgroundType::Pdf;
            }
            _ => {
                return Err(anyhow::anyhow!("Err while parsing `type` attribute of XoppBackground with node id {:?}, is not a valid value", node.id()));
            }
        }

        Ok(())
    }
}

impl XmlWritable for XoppBackground {
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter) {
        w.start_element("background");
        if let Some(name) = self.name.as_ref() {
            w.write_attribute("name", name.as_str());
        }
        self.bg_type.write_to_xml(w);
        w.end_element()
    }
}

/// A Xopp Layer
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct XoppLayer {
    /// Optional layer name
    pub name: Option<String>,
    /// Strokes on this layer
    pub strokes: Vec<XoppStroke>,
    /// Texts on this layer
    pub texts: Vec<XoppText>,
    /// Images on this layer
    pub images: Vec<XoppImage>,
}

impl XmlLoadable for XoppLayer {
    fn load_from_xml(&mut self, node: Node) -> anyhow::Result<()> {
        self.name = node.attribute("name").map(|name| name.to_string());

        for child in node.children() {
            match child.node_type() {
                NodeType::Element => match child.tag_name().name() {
                    "stroke" => {
                        let mut new_stroke = XoppStroke::default();
                        new_stroke.load_from_xml(child)?;
                        self.strokes.push(new_stroke);
                    }
                    "text" => {
                        let mut new_text = XoppText::default();
                        new_text.load_from_xml(child)?;
                        self.texts.push(new_text);
                    }
                    "image" => {
                        let mut new_image = XoppImage::default();
                        new_image.load_from_xml(child)?;
                        self.images.push(new_image);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }
}

impl XmlWritable for XoppLayer {
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter) {
        w.start_element("layer");

        if let Some(name) = self.name.as_ref() {
            w.write_attribute("name", name.as_str());
        }

        for stroke in self.strokes.iter() {
            stroke.write_to_xml(w);
        }
        for text in self.texts.iter() {
            text.write_to_xml(w);
        }
        for image in self.images.iter() {
            image.write_to_xml(w);
        }
        w.end_element();
    }
}

/// A Xopp Color (represented in xml as hex values in format #RRGGBBAA )
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct XoppColor {
    /// red from 0 to 255
    pub red: u8,
    /// green from 0 to 255
    pub green: u8,
    /// blue from 0 to 255
    pub blue: u8,
    /// alpha from 0 to 255
    pub alpha: u8,
}

impl AsXmlAttributeValue for XoppColor {
    fn as_xml_attr_value(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue, self.alpha
        )
    }
}

impl XoppColor {
    /// Parsing from a attribute avlue that is the format #RRGGBBAA
    fn from_hexcolor_attr_value(s: &str) -> Result<Self, anyhow::Error> {
        let s = s.trim().replace("#", "");

        let value = u32::from_str_radix(s.as_str(), 16)?;

        let color = Self {
            red: ((value >> 24) & 0xff) as u8,
            green: ((value >> 16) & 0xff) as u8,
            blue: ((value >> 8) & 0xff) as u8,
            alpha: (value & 0xff) as u8,
        };

        Ok(color)
    }

    /// Parsing from a color attribute value in a stroke
    pub fn from_strokecolor_attr_value(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "black" => Ok(Self {
                red: 0x00,
                green: 0x00,
                blue: 0x00,
                alpha: 0xff,
            }),
            "blue" => Ok(Self {
                red: 0x33,
                green: 0x33,
                blue: 0xcc,
                alpha: 0xff,
            }),
            "red" => Ok(Self {
                red: 0xff,
                green: 0x00,
                blue: 0x00,
                alpha: 0xff,
            }),
            "green" => Ok(Self {
                red: 0x00,
                green: 0x80,
                blue: 0x00,
                alpha: 0xff,
            }),
            "gray" => Ok(Self {
                red: 0x80,
                green: 0x80,
                blue: 0x80,
                alpha: 0xff,
            }),
            "lightblue" => Ok(Self {
                red: 0x80,
                green: 0xc0,
                blue: 0xff,
                alpha: 0xff,
            }),
            "lightgreen" => Ok(Self {
                red: 0x00,
                green: 0xff,
                blue: 0x00,
                alpha: 0xff,
            }),
            "magenta" => Ok(Self {
                red: 0xff,
                green: 0x00,
                blue: 0xff,
                alpha: 0xff,
            }),
            "orange" => Ok(Self {
                red: 0xff,
                green: 0x80,
                blue: 0x00,
                alpha: 0xff,
            }),
            "yellow" => Ok(Self {
                red: 0xff,
                green: 0xff,
                blue: 0xf0,
                alpha: 0xff,
            }),
            "white" => Ok(Self {
                red: 0xff,
                green: 0xff,
                blue: 0xff,
                alpha: 0xff,
            }),
            other => Self::from_hexcolor_attr_value(other),
        }
    }

    /// Parsing from a color attribute value in the background
    fn from_backgroundcolor_attr_value(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "white" => Ok(Self {
                red: 0xff,
                green: 0xff,
                blue: 0xff,
                alpha: 0xff,
            }),
            "blue" => Ok(Self {
                red: 0xa0,
                green: 0xe8,
                blue: 0xff,
                alpha: 0xff,
            }),
            "pink" => Ok(Self {
                red: 0xff,
                green: 0xc0,
                blue: 0xd4,
                alpha: 0xff,
            }),
            "green" => Ok(Self {
                red: 0x80,
                green: 0xff,
                blue: 0xc0,
                alpha: 0xff,
            }),
            "orange" => Ok(Self {
                red: 0xff,
                green: 0xc0,
                blue: 0x80,
                alpha: 0xff,
            }),
            "yellow" => Ok(Self {
                red: 0xff,
                green: 0xff,
                blue: 0x80,
                alpha: 0xff,
            }),
            other => Self::from_hexcolor_attr_value(other),
        }
    }
}

/// Helper to bundle stroke types into one type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XoppStrokeType {
    /// A stroke
    XoppStroke(XoppStroke),
    /// A text
    XoppText(XoppText),
    /// An image
    XoppImage(XoppImage),
}

/// A Xopp Stroke
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct XoppStroke {
    /// the stroke tool
    pub tool: XoppTool,
    /// The stroke color
    pub color: XoppColor,
    /// Stroke fill. None if is not filled, 255 if fully opaque filled
    pub fill: Option<i32>,
    /// The stroke widths.
    /// The first element is the width of the entire stroke, and if existent, every following is a absolute width for the corresponding coordinate.
    /// If they don't exist, the stroke has the first width as constant width
    pub width: Vec<f64>,
    /// The stroke coordinates ( as points where a vec (1.0, 0.0) has length of 1 / 72inch )
    pub coords: Vec<na::Vector2<f64>>,
    /// Optional timestamp
    pub timestamp: Option<u64>,
    /// Optional audio filename
    pub audio_filename: Option<String>,
}

impl XmlLoadable for XoppStroke {
    fn load_from_xml(&mut self, node: Node) -> anyhow::Result<()> {
        match node.attribute("tool").ok_or_else(|| {
            anyhow::anyhow!(
                "failed to parse `tool` attribute in XoppStroke with node id {:?}, could not find attribute",
                node.id()
            )
        })? {
            "pen" => {
                self.tool = XoppTool::Pen;
            }
            "highlighter" => {
                self.tool = XoppTool::Highlighter;
            }
            "eraser" => {
                self.tool = XoppTool::Eraser;
            }
            _ => {}
        }

        self.color =
            XoppColor::from_strokecolor_attr_value(node.attribute("color").ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `color` attribute in XoppStroke with node id {:?}, could not find attribute",
                    node.id()
                )
            })?)?;

        self.fill = if let Some(fill) = node.attribute("fill") {
            Some(fill.parse::<i32>()?)
        } else {
            None
        };

        self.width = node
            .attribute("width")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `width` attribute in XoppStroke with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .split(" ")
            .filter_map(|splitted| splitted.parse::<f64>().ok())
            .collect::<Vec<f64>>();

        self.timestamp = if let Some(_ts) = node.attribute("ts") {
            // the timestamp parsing is fallible and currently not implemented
            // ts.parse::<u64>().ok()
            None
        } else {
            None
        };

        self.audio_filename = node
            .attribute("fn")
            .map(|audio_filename| audio_filename.to_string());

        if let Some(coords) = node.text() {
            let coords = coords
                .split(" ")
                .filter_map(|splitted| splitted.parse::<f64>().ok());

            self.coords = coords
                .clone()
                .zip(coords.clone().skip(1))
                .step_by(2)
                .map(|(x, y)| na::vector![x, y])
                .collect::<Vec<na::Vector2<f64>>>();
        }

        Ok(())
    }
}

impl XmlWritable for XoppStroke {
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter) {
        w.start_element("stroke");
        w.write_attribute("tool", &self.tool.as_xml_attr_value());
        w.write_attribute("color", &self.color.as_xml_attr_value());
        if let Some(fill) = self.fill {
            w.write_attribute("fill", format!("{}", fill).as_str());
        }
        w.write_attribute(
            "width",
            &self
                .width
                .iter()
                .map(|&width| format!("{}", width))
                .collect::<Vec<String>>()
                .join(" "),
        );
        w.write_text(
            &self
                .coords
                .iter()
                .map(|coord| format!("{} {}", coord[0], coord[1]))
                .collect::<Vec<String>>()
                .join(" "),
        );
        w.end_element();
    }
}

/// A xopp stroke tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XoppTool {
    /// the Xopp Pen
    Pen,
    /// the xopp highlighter ( alpha = 0.5 )
    Highlighter,
    /// the xopp eraser
    Eraser,
}

impl AsXmlAttributeValue for XoppTool {
    fn as_xml_attr_value(&self) -> String {
        match self {
            Self::Pen => String::from("pen"),
            Self::Highlighter => String::from("highlighter"),
            Self::Eraser => String::from("eraser"),
        }
    }
}

impl Default for XoppTool {
    fn default() -> Self {
        Self::Pen
    }
}

/// A xopp text
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct XoppText {
    /// The text font
    pub font: String,
    /// The text size
    pub size: u32,
    /// The x position of the upper left corner
    pub x: f64,
    /// The y position of the upper left corner
    pub y: f64,
    /// The text color
    pub color: XoppColor,
    /// The text string
    pub text: String,
}

impl XmlLoadable for XoppText {
    fn load_from_xml(&mut self, node: Node) -> anyhow::Result<()> {
        self.font = node
            .attribute("font")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `font` attribute in XoppText with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .to_string();

        self.size = node
            .attribute("size")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `size` attribute in XoppText with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .parse::<u32>()?;

        self.x = node
            .attribute("x")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `x` attribute in XoppText with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .parse::<f64>()?;

        self.y = node
            .attribute("y")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `y` attribute in XoppText with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .parse::<f64>()?;

        self.color =
            XoppColor::from_strokecolor_attr_value(node.attribute("color").ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `color` attribute in XoppText with node id {:?}, could not find attribute",
                    node.id()
                )
            })?)?;

        if let Some(text) = node.text() {
            self.text = text.to_string();
        }

        Ok(())
    }
}

impl XmlWritable for XoppText {
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter) {
        w.start_element("text");
        w.write_attribute("font", &self.font);
        w.write_attribute("x", &self.x);
        w.write_attribute("y", &self.y);
        w.write_attribute("color", &self.color.as_xml_attr_value());
        w.write_text(&self.text);
        w.end_element();
    }
}

/// a Xopp image
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct XoppImage {
    /// The left x position
    pub left: f64,
    /// The top y position
    pub top: f64,
    /// The right x position
    pub right: f64,
    /// The bottom y position
    pub bottom: f64,
    /// The image data encoded as PNG base64
    pub data: String,
}

impl XmlLoadable for XoppImage {
    fn load_from_xml(&mut self, node: Node) -> anyhow::Result<()> {
        // Left
        self.left = node
            .attribute("left")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `left` attribute in XoppText with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .parse::<f64>()?;

        // Top
        self.top = node
            .attribute("top")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `top` attribute in XoppText with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .parse::<f64>()?;

        // Right
        self.right = node
            .attribute("right")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `right` attribute in XoppText with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .parse::<f64>()?;

        // Bottom
        self.bottom = node
            .attribute("bottom")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "failed to parse `bottom` attribute in XoppText with node id {:?}, could not find attribute",
                    node.id()
                )
            })?
            .parse::<f64>()?;

        // Data
        if let Some(data) = node.text() {
            self.data = data.to_string();
        }

        Ok(())
    }
}

impl XmlWritable for XoppImage {
    fn write_to_xml(&self, w: &mut xmlwriter::XmlWriter) {
        w.start_element("image");
        w.write_attribute("left", &self.left);
        w.write_attribute("top", &self.top);
        w.write_attribute("right", &self.right);
        w.write_attribute("bottom", &self.bottom);
        w.write_text(&self.data);
        w.end_element();
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{FileFormatLoader, FileFormatSaver};
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(pretty_env_logger::init);
    }

    #[test]
    fn load_simple_xopp() -> anyhow::Result<()> {
        setup();
        let to_load = PathBuf::from("./tests/simple.xopp");
        let to_save = PathBuf::from("./temp/simple.json");
        let bytes = std::fs::read(&to_load)?;

        let xopp_format_loader = super::XoppFile::load_from_bytes(&bytes)?;

        // Dumping a json with serde
        let xpp_json = serde_json::to_string_pretty(&xopp_format_loader.xopp_root)?;
        std::fs::write(&to_save, &xpp_json)?;

        Ok(())
    }

    #[test]
    fn load_image_xopp() -> anyhow::Result<()> {
        setup();
        let to_load = PathBuf::from("./tests/image.xopp");
        let to_save = PathBuf::from("./temp/image.json");
        let bytes = std::fs::read(&to_load)?;

        let xopp_format_loader = super::XoppFile::load_from_bytes(&bytes)?;

        // Dumping a json with serde
        let xpp_json = serde_json::to_string_pretty(&xopp_format_loader.xopp_root)?;
        std::fs::write(&to_save, &xpp_json)?;

        Ok(())
    }

    #[test]
    fn load_and_save_simple_xopp() -> anyhow::Result<()> {
        setup();
        let to_load = PathBuf::from("./tests/simple.xopp");
        let to_save = PathBuf::from("./temp/simple-new.xopp");
        let bytes = std::fs::read(&to_load)?;

        let xopp_file = super::XoppFile::load_from_bytes(&bytes)?;

        let xopp_output = xopp_file.save_as_bytes("simple-new.xopp")?;
        std::fs::write(&to_save, &xopp_output)?;

        Ok(())
    }

    #[test]
    fn load_and_save_image_xopp() -> anyhow::Result<()> {
        setup();
        let to_load = PathBuf::from("./tests/image.xopp");
        let to_save = PathBuf::from("./temp/image-new.xopp");
        let bytes = std::fs::read(&to_load)?;

        let xopp_file = super::XoppFile::load_from_bytes(&bytes)?;

        let xopp_output = xopp_file.save_as_bytes("image-new.xopp")?;
        std::fs::write(&to_save, &xopp_output)?;

        Ok(())
    }
}
