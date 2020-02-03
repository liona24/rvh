pub type Range = (u32, u32);
pub type Mask = u32;
pub type Field = u32;
pub type Priority = u32;

pub trait Rule: PartialEq {
    fn priority(&self) -> Priority;
    fn masks(&self) -> &[Mask];
    fn fields(&self) -> &[Field];
}
pub trait Packet {
    fn fields(&self) -> &[Field];
}

#[cfg(test)]
pub(crate) mod mocks {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct MockRule {
        fields: Vec<Field>,
        masks: Vec<Mask>,
        priority: Priority,
    }

    impl MockRule {
        pub fn new(fields: Vec<Field>, masks: Vec<Mask>, priority: Priority) -> Self {
            Self {
                fields,
                masks,
                priority,
            }
        }
    }

    impl Rule for MockRule {
        fn fields(&self) -> &[Field] {
            &self.fields
        }
        fn masks(&self) -> &[Mask] {
            &self.masks
        }
        fn priority(&self) -> Priority {
            self.priority
        }
    }

    impl PartialEq for MockRule {
        fn eq(&self, other: &Self) -> bool {
            self.priority() == other.priority()
        }
    }

    #[derive(Debug, Clone)]
    pub struct MockPacket {
        fields: Vec<Field>,
    }

    impl MockPacket {
        pub fn new(fields: Vec<Field>) -> Self {
            Self { fields }
        }
    }

    impl Packet for MockPacket {
        fn fields(&self) -> &[Field] {
            &self.fields
        }
    }
}
