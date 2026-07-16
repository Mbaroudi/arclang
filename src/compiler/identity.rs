//! Stable element identity.
//!
//! Every model element gets a UUID that is DETERMINISTIC: the same element
//! (same kind + same user-visible id) always produces the same UUID, across
//! machines and compilations. This is the foundation for diff, merge, and
//! Capella round-trip.
//!
//! Rules:
//! - The UUID is a v5 (SHA-1) UUID in the ArcLang namespace, derived from
//!   `"{kind}:{id}"`.
//! - `id` is the user-visible identifier: the explicit `id:` attribute when
//!   present, otherwise the element's qualified name. Renaming an element
//!   without an explicit `id:` therefore changes its UUID — declare `id:` on
//!   anything you want to track across renames.

use uuid::Uuid;

/// Fixed ArcLang namespace: uuid5(NAMESPACE_URL, "https://arclang.org/model")
/// = febb6e9d-b5a0-51d7-bb17-0e4e67346213.
/// Never change this value — it would re-identify every element everywhere.
const ARCLANG_NAMESPACE: Uuid = Uuid::from_bytes([
    0xfe, 0xbb, 0x6e, 0x9d, 0xb5, 0xa0, 0x51, 0xd7, 0xbb, 0x17, 0x0e, 0x4e, 0x67, 0x34, 0x62,
    0x13,
]);

/// Deterministic UUID for a model element.
///
/// `kind` is the element category ("requirement", "component", "function",
/// "actor", "trace", ...); `id` is the user-visible identifier.
pub fn element_uuid(kind: &str, id: &str) -> String {
    let name = format!("{}:{}", kind, id);
    Uuid::new_v5(&ARCLANG_NAMESPACE, name.as_bytes()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_element_same_uuid() {
        assert_eq!(
            element_uuid("component", "LC-001"),
            element_uuid("component", "LC-001")
        );
    }

    #[test]
    fn different_kind_different_uuid() {
        assert_ne!(
            element_uuid("component", "LC-001"),
            element_uuid("function", "LC-001")
        );
    }

    #[test]
    fn different_id_different_uuid() {
        assert_ne!(
            element_uuid("component", "LC-001"),
            element_uuid("component", "LC-002")
        );
    }

    #[test]
    fn uuid_is_stable_across_versions() {
        // Golden value: if this test breaks, element identity broke globally.
        assert_eq!(
            element_uuid("component", "LC-001"),
            "420f95c6-ce14-521c-a5ac-f1772cc9f0a3"
        );
    }
}
