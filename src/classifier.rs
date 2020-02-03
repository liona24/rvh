use crate::range_vector_hash_map::RVHashMap;
use crate::types::*;

#[derive(Debug, Clone)]
pub struct RVHClassifier<R: Rule> {
    hash_maps: Vec<RVHashMap<R>>,
}

impl<R: Rule> RVHClassifier<R> {
    pub fn new(ranges: impl Iterator<Item = Vec<Range>>) -> Self {
        let mut hash_maps = Vec::new();
        for range in ranges {
            hash_maps.push(RVHashMap::new(range));
        }

        Self { hash_maps }
    }

    pub fn add_rule(&mut self, rule: R) -> bool {
        for hm in self.hash_maps.iter_mut() {
            if hm.can_insert(&rule) {
                if hm.insert(rule) {
                    self.sort_hash_maps();
                    return true;
                }

                // this only happens if the priority of `rule` is not unique
                break;
            }
        }
        false
    }

    pub fn remove_rule(&mut self, rule: &R) -> bool {
        for hm in self.hash_maps.iter_mut() {
            if hm.remove(&rule) {
                self.sort_hash_maps();
                return true;
            }
        }
        false
    }

    pub fn classify(&self, p: &impl Packet) -> Option<&R> {
        let mut highest_matching_priority = 0;
        let mut best_match = None;

        for hm in self.hash_maps.iter() {
            if hm.highest_priority() < highest_matching_priority {
                break;
            }

            if let Some(matching_rule) = hm.check_match(p) {
                if matching_rule.priority() > highest_matching_priority {
                    highest_matching_priority = matching_rule.priority();
                    best_match = Some(matching_rule);
                }
            }
        }

        best_match
    }

    fn sort_hash_maps(&mut self) {
        self.hash_maps
            .sort_by(|a, b| b.highest_priority().cmp(&a.highest_priority()));
    }
}

impl<R: Rule> Default for RVHClassifier<R> {
    fn default() -> Self {
        panic!("Not implemented!");
        // TODO this should return the standard split for 5-Tuples
        // Self::new(vec![vec![], vec![], vec![], vec![]].into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::mocks::{MockPacket, MockRule};

    #[test]
    fn test_insertions_keep_correct_order_of_hash_tables() {
        let mut rvh = RVHClassifier::<MockRule>::new(
            vec![
                vec![(3, 6), (0, 3)],
                vec![(0, 3), (3, 6)],
                vec![(0, 3), (0, 3)],
                vec![(3, 6), (3, 6)],
            ]
            .into_iter(),
        );

        let r11 = MockRule::new(vec![0b1, 0b10], vec![0b11, 0b1], 1);
        assert!(rvh.add_rule(r11));

        let r21 = MockRule::new(vec![0b1, 0b10], vec![0b111, 0b1], 3);
        assert!(rvh.add_rule(r21));

        assert_eq!(rvh.hash_maps[0].highest_priority(), 3);
        assert_eq!(rvh.hash_maps[1].highest_priority(), 1);

        let r31 = MockRule::new(vec![0b1, 0b10], vec![0b11, 0b111], 5);
        assert!(rvh.add_rule(r31));

        assert_eq!(rvh.hash_maps[0].highest_priority(), 5);

        let r41 = MockRule::new(vec![0b1, 0b10], vec![0b111, 0b11_111], 7);
        assert!(rvh.add_rule(r41));

        assert_eq!(rvh.hash_maps[0].highest_priority(), 7);
        assert_eq!(rvh.hash_maps[1].highest_priority(), 5);
        assert_eq!(rvh.hash_maps[2].highest_priority(), 3);
        assert_eq!(rvh.hash_maps[3].highest_priority(), 1);
    }

    #[test]
    fn test_removals_keep_correct_order_of_hash_tables() {
        let mut rvh = RVHClassifier::<MockRule>::new(
            vec![
                vec![(3, 6), (0, 3)],
                vec![(0, 3), (3, 6)],
                vec![(0, 3), (0, 3)],
                vec![(3, 6), (3, 6)],
            ]
            .into_iter(),
        );

        let r11 = MockRule::new(vec![0b1, 0b10], vec![0b11, 0b1], 1);
        assert!(rvh.add_rule(r11.clone()));

        let r21 = MockRule::new(vec![0b1, 0b10], vec![0b111, 0b1], 3);
        assert!(rvh.add_rule(r21.clone()));

        let r31 = MockRule::new(vec![0b1, 0b10], vec![0b11, 0b111], 5);
        assert!(rvh.add_rule(r31.clone()));

        let r41 = MockRule::new(vec![0b1, 0b10], vec![0b111, 0b11_111], 7);
        assert!(rvh.add_rule(r41.clone()));

        assert!(rvh.remove_rule(&r31));
        assert_eq!(rvh.hash_maps[0].highest_priority(), 7);
        assert_eq!(rvh.hash_maps[1].highest_priority(), 3);

        assert!(rvh.remove_rule(&r41));
        assert_eq!(rvh.hash_maps[0].highest_priority(), 3);
        assert_eq!(rvh.hash_maps[1].highest_priority(), 1);
    }

    #[test]
    fn test_add_rule_inserts_rule_into_correct_table() {
        let mut rvh = RVHClassifier::<MockRule>::new(
            vec![
                vec![(0, 3), (0, 3)],
                vec![(3, 6), (0, 3)],
                vec![(0, 3), (3, 6)],
                vec![(3, 6), (3, 6)],
            ]
            .into_iter(),
        );

        let r11 = MockRule::new(vec![0b1, 0b10], vec![0b11, 0b1], 1);
        let r12 = MockRule::new(vec![0b1, 0b10], vec![0b1, 0b11], 2);
        assert!(rvh.add_rule(r11));
        assert!(rvh.add_rule(r12));

        let r21 = MockRule::new(vec![0b1, 0b10], vec![0b111, 0b1], 3);
        let r22 = MockRule::new(vec![0b1, 0b10], vec![0b11_111, 0b11], 4);
        assert!(rvh.add_rule(r21));
        assert!(rvh.add_rule(r22));

        let r31 = MockRule::new(vec![0b1, 0b10], vec![0b11, 0b111], 5);
        let r32 = MockRule::new(vec![0b1, 0b10], vec![0b1, 0b11_111], 6);
        assert!(rvh.add_rule(r31));
        assert!(rvh.add_rule(r32));

        let r41 = MockRule::new(vec![0b1, 0b10], vec![0b111, 0b11_111], 7);
        let r42 = MockRule::new(vec![0b1, 0b10], vec![0b11_111, 0b111], 8);
        assert!(rvh.add_rule(r41));
        assert!(rvh.add_rule(r42));

        let prios: Vec<_> = rvh.hash_maps[0].priorities.iter().collect();
        assert_eq!(prios, vec![&7, &8]);

        let prios: Vec<_> = rvh.hash_maps[1].priorities.iter().collect();
        assert_eq!(prios, vec![&5, &6]);

        let prios: Vec<_> = rvh.hash_maps[2].priorities.iter().collect();
        assert_eq!(prios, vec![&3, &4]);

        let prios: Vec<_> = rvh.hash_maps[3].priorities.iter().collect();
        assert_eq!(prios, vec![&1, &2]);
    }

    #[test]
    fn test_classifier_classifies_correctly() {
        let mut rvh = RVHClassifier::<MockRule>::new(
            vec![
                vec![(0, 3)],
                vec![(3, 6)],
                vec![(6, 9)],
            ]
            .into_iter(),
        );

        let r11 = MockRule::new(vec![0b11], vec![0b11], 1);
        let r12 = MockRule::new(vec![0b1], vec![0b1], 3);
        rvh.add_rule(r11);
        rvh.add_rule(r12);

        let r21 = MockRule::new(vec![0b100], vec![0b111], 4);
        let r22 = MockRule::new(vec![0b101], vec![0b1_1111], 2);
        rvh.add_rule(r21);
        rvh.add_rule(r22);

        let r31 = MockRule::new(vec![0b11_1001], vec![0b11_1111], 6);
        let r32 = MockRule::new(vec![0b11_1100], vec![0b1111_1111], 5);
        rvh.add_rule(r31);
        rvh.add_rule(r32);

        let p31 = MockPacket::new(vec![0b11_1001]); // matches r12, r31
        let p32 = MockPacket::new(vec![0b11_1100]); // matches r21, r32

        let p12 = MockPacket::new(vec![0b00_101]); // matches r22, r12
        let p21 = MockPacket::new(vec![0b00_100]); // matches r21

        let p_none = MockPacket::new(vec![0b0]);

        assert_eq!(rvh.classify(&p31).expect("should match").priority(), 6);
        assert_eq!(rvh.classify(&p32).expect("should match").priority(), 5);
        assert_eq!(rvh.classify(&p12).expect("should match").priority(), 3);
        assert_eq!(rvh.classify(&p21).expect("should match").priority(), 4);
        assert!(rvh.classify(&p_none).is_none());
    }
}
