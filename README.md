rvh - Range-Vector Hash Packet Classification
=============================================

Implementation of the algorithm described in [RVH: Range-Vector Hash for Fast Online Packet Classification](https://arxiv.org/abs/1909.07159)


## Quick Reference

```rust
use rvh::RVHClassifier;
use rvh::types::{Field, Mask, Priority, Rule, Packet};
// or simply use rvh::prelude::*;

#[derive(Clone)]
struct MyRule {
    fields: Vec<Field>,
    masks: Vec<Mask>,
    priority: Priority,
}

impl Rule for MyRule {
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

impl PartialEq for MyRule {
    fn eq(&self, other: &Self) -> bool {
        self.priority() == other.priority()
    }
}

struct MyPacket {
    fields: Vec<Field>
}

impl Packet for MyPacket {
    fn fields(&self) -> &[Field] {
        &self.fields
    }
}

fn main() {
    // construct a classifier which uses `MyRule` for packets with 2 fields
    // all rules are distributed into four hash tables, each associated with the given mask lengths
    // f.e. [(3, 6), (0, 3)] gets all rules which have mask length 3 up to 6 (exclusive) bits on the first field
    // and 0 up to 3 bits on the second field.
    let mut classifier = RVHClassifier::<MyRule>::new(
        vec![
            vec![(3, 6), (0, 3)],
            vec![(0, 3), (3, 6)],
            vec![(0, 3), (0, 3)],
            vec![(3, 6), (3, 6)],
        ]
        .into_iter(),
    );

    // add a rule
    // this rule will match packets (0101*, 11*)
    let rule = MyRule {
        fields: vec![0b0101, 0b11],
        masks: vec![0b1111, 0b11],
        priority: 1
    };
    classifier.add_rule(rule.clone());

    // let's try to match a packet
    let packet = MyPacket { fields: vec![0b11_0101, 0b101_0011] };
    assert_eq!(classifier.classify(&packet).expect("Should match"), rule);

    // and finally remove the rule, just for fun
    classifier.remove_rule(&rule);
}
```
