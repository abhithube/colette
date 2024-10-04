use std::fmt::Write;

use sea_query::Iden;

pub(crate) struct JsonbArrayElements;

impl Iden for JsonbArrayElements {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "JSONB_ARRAY_ELEMENTS").unwrap();
    }
}

pub(crate) struct JsonbBuildObject;

impl Iden for JsonbBuildObject {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "JSONB_BUILD_OBJECT").unwrap();
    }
}
