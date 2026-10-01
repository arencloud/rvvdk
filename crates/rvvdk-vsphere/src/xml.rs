use crate::{Error, Result};
use roxmltree::{Document, Node, ParsingOptions};

pub(crate) const VIM: &str = "urn:vim25";
const SOAP: &str = "http://schemas.xmlsoap.org/soap/envelope/";
pub(crate) const XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";
pub(crate) const RESPONSE_LIMIT: usize = 2 * 1024 * 1024;

pub(crate) fn escape(value: &str) -> Result<String> {
    if value.len() > 4096 || value.chars().any(|c| !matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')) {
        return Err(Error::InvalidInput);
    }
    Ok(value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;"))
}

pub(crate) fn parse(raw: &str) -> Result<Document<'_>> {
    if raw.len() > RESPONSE_LIMIT {
        return Err(Error::ResponseLimit);
    }
    // Reject even empty DTDs (which roxmltree otherwise accepts). UTF-8 only.
    if raw.contains("<!DOCTYPE") || raw.contains("<!ENTITY") {
        return Err(Error::Xml);
    }
    let doc = Document::parse_with_options(
        raw,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: 16_384,
            ..Default::default()
        },
    )
    .map_err(|_| Error::Xml)?;
    for node in doc.descendants() {
        if node.ancestors().take(66).count() > 64
            || node.text().is_some_and(|s| s.len() > 64 * 1024)
        {
            return Err(Error::Xml);
        }
    }
    Ok(doc)
}

pub(crate) fn response<'a, 'i>(doc: &'a Document<'i>, method: &str) -> Result<Node<'a, 'i>> {
    let envelope = doc.root_element();
    if !envelope.has_tag_name((SOAP, "Envelope")) {
        return Err(Error::Schema);
    }
    let mut bodies = envelope
        .children()
        .filter(|n| n.has_tag_name((SOAP, "Body")));
    let body = bodies.next().ok_or(Error::Schema)?;
    if bodies.next().is_some() {
        return Err(Error::Schema);
    }
    let mut children = body.children().filter(Node::is_element);
    let result = children.next().ok_or(Error::Schema)?;
    if children.next().is_some() {
        return Err(Error::Schema);
    }
    if result.has_tag_name((SOAP, "Fault")) {
        let kind = result
            .children()
            .find(|n| n.tag_name().name() == "detail")
            .and_then(|n| n.children().find(Node::is_element));
        return Err(match kind.map(|n| n.tag_name().name()) {
            Some("InvalidLoginFault") => Error::InvalidLogin,
            Some("NotAuthenticatedFault") => Error::NotAuthenticated,
            Some("NoPermissionFault") => Error::NoPermission,
            Some("RestrictedVersionFault" | "NotSupportedFault") => Error::Restricted,
            _ => Error::SoapFault,
        });
    }
    if !result.has_tag_name((VIM, format!("{method}Response").as_str())) {
        return Err(Error::Schema);
    }
    Ok(result)
}

pub(crate) fn child<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Result<Option<Node<'a, 'i>>> {
    let mut nodes = node.children().filter(|n| n.has_tag_name((VIM, name)));
    let first = nodes.next();
    if nodes.next().is_some() {
        return Err(Error::Schema);
    }
    Ok(first)
}

pub(crate) fn required<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Result<Node<'a, 'i>> {
    child(node, name)?.ok_or(Error::Schema)
}

pub(crate) fn text<'a>(node: Node<'a, '_>) -> Result<&'a str> {
    if node.children().any(|n| n.is_element()) {
        return Err(Error::Schema);
    }
    node.text().ok_or(Error::Schema)
}

pub(crate) fn scalar(node: Node<'_, '_>) -> Result<String> {
    let value = text(node)?;
    if value.len() > 100
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_. -".contains(&b))
    {
        return Err(Error::Schema);
    }
    Ok(value.to_owned())
}

pub(crate) fn number(node: Node<'_, '_>) -> Result<u64> {
    text(node)?.parse().map_err(|_| Error::Schema)
}

pub(crate) fn boolean(node: Node<'_, '_>) -> Result<bool> {
    match text(node)? {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(Error::Schema),
    }
}

// roxmltree's attribute("type") matches by local name across namespaces.
// ManagedObjectReference.type must specifically be the unqualified attribute.
pub(crate) fn reference_type<'a>(node: Node<'a, '_>) -> Option<&'a str> {
    node.attributes()
        .find(|a| a.namespace().is_none() && a.name() == "type")
        .map(|a| a.value())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escaping_and_invalid_characters() {
        assert_eq!(escape("a<&\"'>").unwrap(), "a&lt;&amp;&quot;&apos;&gt;");
        assert_eq!(escape("\0"), Err(Error::InvalidInput));
        assert!(escape(&"x".repeat(4097)).is_err());
    }
    #[test]
    fn hostile_xml_is_bounded() {
        for xml in [
            "<!DOCTYPE a><a/>".to_owned(),
            "<!DOCTYPE a [<!ENTITY x 'x'>]><a>&x;</a>".to_owned(),
            "<a>".repeat(65) + &"</a>".repeat(65),
            "<a>".to_owned() + &"x".repeat(65537) + "</a>",
            "<a>".to_owned() + &"<b/>".repeat(16384) + "</a>",
            "<a/>junk".to_owned(),
        ] {
            assert!(parse(&xml).is_err());
        }
        assert!(matches!(
            parse(&"x".repeat(RESPONSE_LIMIT + 1)),
            Err(Error::ResponseLimit)
        ));
    }
    #[test]
    fn namespaces_shape_and_faults() {
        let wrap = |body: &str| {
            format!(
                "<s:Envelope xmlns:s='{SOAP}' xmlns:v='{VIM}'><s:Body>{body}</s:Body></s:Envelope>"
            )
        };
        for body in [
            "<LoginResponse/>",
            "<v:LogoutResponse/>",
            "<v:LoginResponse/><v:LoginResponse/>",
        ] {
            assert_eq!(
                response(&parse(&wrap(body)).unwrap(), "Login"),
                Err(Error::Schema)
            );
        }
        let raw = wrap(
            "<s:Fault><faultstring>SECRET</faultstring><detail><v:InvalidLoginFault/></detail></s:Fault>",
        );
        assert_eq!(
            response(&parse(&raw).unwrap(), "Login"),
            Err(Error::InvalidLogin)
        );
        assert!(!format!("{:?}", response(&parse(&raw).unwrap(), "Login")).contains("SECRET"));
    }
}
