//! The static UPnP description documents of the MediaRenderer.
//!
//! WHY: a control point learns everything about a device from two
//! documents: the *device description* (`/description.xml`) and one
//! *service description* (SCPD) per service. A renderer that serves a
//! device description but no SCPD is discovered and then ignored, because
//! the control point does not know which actions the services offer.
//!
//! HOW: the action list and the state variables live in exactly one table
//! in this module. The SOAP dispatcher matches on the same names, and a
//! unit test walks the table to prove that every advertised action is
//! actually implemented — a control point that calls an advertised action
//! and gets "invalid action" is a broken renderer.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_dlna::description::{device_description, service_description};
//!
//! let document = device_description(
//!     "Living Room",
//!     "uuid:abc",
//!     "http://10.0.0.5:8200/",
//!     "PhairPlayRust",
//!     "Receiver",
//! );
//! assert!(document.contains("urn:schemas-upnp-org:device:MediaRenderer:1"));
//!
//! let scpd = service_description("AVTransport");
//! assert!(scpd.contains("<name>Play</name>"));
//! ```

mod arguments;
mod tables;

use tables::SERVICES;

/// The name of the ConnectionManager service.
pub const SERVICE_CONNECTION_MANAGER: &str = "ConnectionManager";

/// The name of the AVTransport service.
pub const SERVICE_AV_TRANSPORT: &str = "AVTransport";

/// The name of the RenderingControl service.
pub const SERVICE_RENDERING_CONTROL: &str = "RenderingControl";

/// The UPnP device type of a media renderer.
pub const MEDIA_RENDERER_DEVICE_TYPE: &str = "urn:schemas-upnp-org:device:MediaRenderer:1";

/// The control path of the ConnectionManager service.
pub const CONTROL_PATH_CONNECTION_MANAGER: &str = "/upnp/control/ConnectionManager";

/// The control path of the AVTransport service.
pub const CONTROL_PATH_AV_TRANSPORT: &str = "/upnp/control/AVTransport";

/// The control path of the RenderingControl service.
pub const CONTROL_PATH_RENDERING_CONTROL: &str = "/upnp/control/RenderingControl";

/// The event path of the ConnectionManager service.
pub const EVENT_PATH_CONNECTION_MANAGER: &str = "/upnp/event/ConnectionManager";

/// The event path of the AVTransport service.
pub const EVENT_PATH_AV_TRANSPORT: &str = "/upnp/event/AVTransport";

/// The event path of the RenderingControl service.
pub const EVENT_PATH_RENDERING_CONTROL: &str = "/upnp/event/RenderingControl";

/// The names of every action of every service.
///
/// WHY: the SOAP dispatcher and the SCPD must agree. This list is the
/// single source of truth and the unit test at the bottom of this module
/// proves that the dispatcher handles every entry.
#[must_use]
pub fn advertised_actions(service: &str) -> Vec<&'static str> {
    SERVICES
        .iter()
        .find(|spec| spec.name == service)
        .map(|spec| spec.actions.iter().map(|action| action.name).collect())
        .unwrap_or_default()
}

/// The names of every service of the renderer.
#[must_use]
pub fn service_names() -> Vec<&'static str> {
    SERVICES.iter().map(|spec| spec.name).collect()
}

/// Escapes a text for use in an XML document.
#[must_use]
pub fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
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

/// Builds the device description document.
#[must_use]
pub fn device_description(
    friendly_name: &str,
    udn: &str,
    presentation_url: &str,
    manufacturer: &str,
    model: &str,
) -> String {
    let mut out = String::with_capacity(2048);
    out.push_str("<?xml version=\"1.0\"?>\n");
    out.push_str("<root xmlns=\"urn:schemas-upnp-org:device-1-0\">\n");
    out.push_str("  <specVersion><major>1</major><minor>0</minor></specVersion>\n");
    out.push_str("  <device>\n");
    out.push_str(&format!(
        "    <deviceType>{MEDIA_RENDERER_DEVICE_TYPE}</deviceType>\n"
    ));
    out.push_str(&format!(
        "    <friendlyName>{}</friendlyName>\n",
        escape_xml(friendly_name)
    ));
    out.push_str(&format!(
        "    <manufacturer>{}</manufacturer>\n",
        escape_xml(manufacturer)
    ));
    out.push_str(
        "    <manufacturerURL>https://github.com/mazer666/PhairPlayRust</manufacturerURL>\n",
    );
    out.push_str("    <modelDescription>PhairPlayRust DLNA MediaRenderer</modelDescription>\n");
    out.push_str(&format!(
        "    <modelName>{}</modelName>\n",
        escape_xml(model)
    ));
    out.push_str("    <modelNumber>1</modelNumber>\n");
    out.push_str("    <modelURL>https://github.com/mazer666/PhairPlayRust</modelURL>\n");
    out.push_str(&format!("    <UDN>{}</UDN>\n", escape_xml(udn)));
    out.push_str(&format!(
        "    <presentationURL>{}</presentationURL>\n",
        escape_xml(presentation_url)
    ));
    out.push_str("    <serviceList>\n");
    for spec in SERVICES {
        out.push_str(&service_entry(spec.name));
    }
    out.push_str("    </serviceList>\n");
    out.push_str("  </device>\n");
    out.push_str("</root>\n");
    out
}

/// Builds the `<service>` entry of one service.
fn service_entry(name: &str) -> String {
    format!(
        "      <service>\n\
         \x20       <serviceType>urn:schemas-upnp-org:service:{name}:1</serviceType>\n\
         \x20       <serviceId>urn:upnp-org:serviceId:{name}</serviceId>\n\
         \x20       <SCPDURL>/{name}.xml</SCPDURL>\n\
         \x20       <controlURL>/upnp/control/{name}</controlURL>\n\
         \x20       <eventSubURL>/upnp/event/{name}</eventSubURL>\n\
         \x20     </service>\n"
    )
}

/// Builds the service description (SCPD) of one service.
///
/// # Panics
/// Never — an unknown service name yields an empty string, which the
/// caller turns into a "not found" answer.
#[must_use]
pub fn service_description(service: &str) -> String {
    let Some(spec) = SERVICES.iter().find(|spec| spec.name == service) else {
        return String::new();
    };
    let mut out = String::with_capacity(2048);
    out.push_str("<?xml version=\"1.0\"?>\n");
    out.push_str("<scpd xmlns=\"urn:schemas-upnp-org:service-1-0\">\n");
    out.push_str("  <specVersion><major>1</major><minor>0</minor></specVersion>\n");
    out.push_str("  <actionList>\n");
    for action in spec.actions {
        out.push_str("    <action>\n");
        out.push_str(&format!("      <name>{}</name>\n", action.name));
        if !action.arguments.is_empty() {
            out.push_str("      <argumentList>\n");
            for argument in action.arguments {
                out.push_str("        <argument>\n");
                out.push_str(&format!("          <name>{}</name>\n", argument.name));
                out.push_str(&format!(
                    "          <direction>{}</direction>\n",
                    argument.direction
                ));
                out.push_str(&format!(
                    "          <relatedStateVariable>{}</relatedStateVariable>\n",
                    argument.variable
                ));
                out.push_str("        </argument>\n");
            }
            out.push_str("      </argumentList>\n");
        }
        out.push_str("    </action>\n");
    }
    out.push_str("  </actionList>\n");
    out.push_str("  <serviceStateTable>\n");
    for variable in spec.variables {
        if variable.send_events {
            out.push_str("    <stateVariable sendEvents=\"yes\">\n");
        } else {
            out.push_str("    <stateVariable sendEvents=\"no\">\n");
        }
        out.push_str(&format!("      <name>{}</name>\n", variable.name));
        out.push_str(&format!(
            "      <dataType>{}</dataType>\n",
            variable.data_type
        ));
        if !variable.allowed.is_empty() {
            out.push_str("      <allowedValueList>\n");
            for value in variable.allowed {
                out.push_str(&format!("        <allowedValue>{value}</allowedValue>\n"));
            }
            out.push_str("      </allowedValueList>\n");
        }
        out.push_str("    </stateVariable>\n");
    }
    out.push_str("  </serviceStateTable>\n");
    out.push_str("</scpd>\n");
    out
}

#[cfg(test)]
mod tests;
