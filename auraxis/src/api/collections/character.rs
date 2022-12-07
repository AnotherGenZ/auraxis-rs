use crate::api::collections::Collection;

pub struct CharacterCollection {}

impl Collection for CharacterCollection {
    fn name() -> &'static str {
        "character"
    }
}
