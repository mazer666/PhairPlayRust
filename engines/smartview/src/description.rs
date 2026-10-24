//! The description document of the Smart View receiver.
//!
//! WHAT: the document that the `LOCATION` header of the SSDP answer points
//! at. It carries the facts a sender needs before it talks to the receiver:
//! the display name, the model, the base URL of the API and the
//! capabilities of the receiver.
//!
//! WHY: Samsung documents the second step of the discovery: "the sender
//! fetches the description document at the location and reads the display
//! name out of it". A receiver whose document is empty, or whose document
//! carries a name that a parser cannot read, is found and then dropped —
//! the device appears in the list of the phone and vanishes when the user
//! taps it.
//!
//! HOW: the document is a small, static file, so it is built by a string
//! builder and not by an XML tree. Every text that goes into the document
//! is sanitised and escaped: a name that contains a control character must
//! not produce a document that a parser rejects.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::description::DeviceDescription;
//!
//! let description = DeviceDescription::new(
//!     "Living Room",
//!     "PhairPlay Rust Receiver",
//!     "uuid:abc",
//!     "http://10.0.0.5:8001/api/v2/",
//! );
//! let document = description.to_xml();
//! assert!(document.contains("<friendlyName>Living Room</friendlyName>"));
//! assert!(document.contains("<baseURL>http://10.0.0.5:8001/api/v2/</baseURL>"));
//! ```

use phairplay_core::capabilities::MediaCapabilities;

/// The namespace of the description document.
///
/// WHY: the search target of the MultiScreen service is the only public
/// name of the service. Using it as the namespace of the document keeps
/// the two together — a reader that knows the target can parse the
/// document without a second table.
pub const NAMESPACE: &str = "urn:samsung.com:service:MultiScreenService:1";

/// The path the description document is served under.
pub const DESCRIPTION_PATH: &str = "/description.xml";

/// The facts of the receiver that the description document carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceDescription {
    /// The name the sender shows in its device list.
    pub friendly_name: String,
    /// The manufacturer of the receiver.
    pub manufacturer: String,
    /// The model of the receiver.
    pub model_name: String,
    /// The model number of the receiver.
    pub model_number: String,
    /// The unique device name of the receiver.
    pub udn: String,
    /// The base URL of the API of the receiver.
    pub base_url: String,
    /// The video codecs the receiver can decode, as MIME types.
    pub video_codecs: Vec<String>,
    /// The audio codecs the receiver can decode, as MIME types.
    pub audio_codecs: Vec<String>,
}

impl DeviceDescription {
    /// A description with the given facts and without any capability.
    #[must_use]
    pub fn new(
        friendly_name: impl Into<String>,
        model_name: impl Into<String>,
        udn: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            friendly_name: friendly_name.into(),
            manufacturer: "PhairPlayRust".to_string(),
            model_name: model_name.into(),
            model_number: "1".to_string(),
            udn: udn.into(),
            base_url: base_url.into(),
            video_codecs: Vec::new(),
            audio_codecs: Vec::new(),
        }
    }

    /// Adds the media capabilities of the receiver.
    ///
    /// WHY: a receiver must only advertise what it can decode. The
    /// capability set comes from the platform layer — the engine never
    /// queries the decoders itself.
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: &MediaCapabilities) -> Self {
        self.video_codecs = capabilities
            .video
            .iter()
            .map(|codec| codec.mime_type().to_string())
            .collect();
        self.audio_codecs = capabilities
            .audio
            .iter()
            .map(|codec| codec.mime_type().to_string())
            .collect();
        self
    }

    /// The document as an XML string.
    #[must_use]
    pub fn to_xml(&self) -> String {
        description_document(self)
    }
}

/// Builds the description document of a receiver.
#[must_use]
pub fn description_document(description: &DeviceDescription) -> String {
    let mut out = String::with_capacity(2048);
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str(&format!("<root xmlns=\"{NAMESPACE}\">\n"));
    out.push_str("  <specVersion><major>1</major><minor>0</minor></specVersion>\n");
    out.push_str("  <device>\n");
    out.push_str(&format!(
        "    <friendlyName>{}</friendlyName>\n",
        escape_xml(&description.friendly_name)
    ));
    out.push_str(&format!(
        "    <manufacturer>{}</manufacturer>\n",
        escape_xml(&description.manufacturer)
    ));
    out.push_str(&format!(
        "    <modelName>{}</modelName>\n",
        escape_xml(&description.model_name)
    ));
    out.push_str(&format!(
        "    <modelNumber>{}</modelNumber>\n",
        escape_xml(&description.model_number)
    ));
    out.push_str(&format!(
        "    <UDN>{}</UDN>\n",
        escape_xml(&description.udn)
    ));
    out.push_str(&format!(
        "    <baseURL>{}</baseURL>\n",
        escape_xml(&description.base_url)
    ));
    out.push_str(&format!("    <serviceType>{NAMESPACE}</serviceType>\n"));
    out.push_str("    <capabilities>\n");
    for codec in &description.video_codecs {
        out.push_str(&format!(
            "      <capability kind=\"video\" value=\"{}\"/>\n",
            escape_xml(codec)
        ));
    }
    for codec in &description.audio_codecs {
        out.push_str(&format!(
            "      <capability kind=\"audio\" value=\"{}\"/>\n",
            escape_xml(codec)
        ));
    }
    out.push_str("    </capabilities>\n");
    out.push_str("  </device>\n");
    out.push_str("</root>\n");
    out
}

/// Escapes a text for use in an XML document.
#[must_use]
pub fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in sanitize_xml_text(text).chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

/// Removes the characters that XML 1.0 does not allow in a text.
///
/// WHY: XML 1.0 forbids every control character except the tabulator, the
/// line feed and the carriage return — and it has no escape for them. A
/// device name that carries one would therefore produce a document that no
/// parser accepts. The characters are removed instead: a sender that shows
/// a slightly shorter name is a smaller problem than a sender that cannot
/// read the document at all.
#[must_use]
pub fn sanitize_xml_text(text: &str) -> String {
    text.chars()
        .filter(|character| {
            let code = *character as u32;
            matches!(
                code,
                0x09 | 0x0A | 0x0D | 0x20..=0xD7FF | 0xE000..=0xFFFD
            )
        })
        .collect()
}

#[cfg(test)]
mod tests;
