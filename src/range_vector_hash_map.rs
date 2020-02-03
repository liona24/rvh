use std::collections::{BTreeSet, HashMap};

use crate::types::*;

fn get_masks<'a, I: Iterator<Item = &'a Range>>(ranges: I) -> Vec<Mask> {
    ranges
        .map(|r| {
            let mut m = 0;
            for _ in 0..r.0 {
                m <<= 1;
                m |= 1;
            }

            m
        })
        .collect()
}

#[inline]
fn is_match(field1: Field, field2: Field, mask: Mask) -> bool {
    ((field1 ^ field2) & mask) == 0
}

#[derive(Debug, Clone)]
pub(crate) struct RVHashMap<R: Rule> {
    pub(crate) highest_priority: Priority,
    pub(crate) priorities: BTreeSet<Priority>,
    pub(crate) masks: Vec<Mask>,
    pub(crate) ranges: Vec<Range>,
    pub(crate) hash_map: HashMap<u32, Vec<R>>,
}

impl<R: Rule> RVHashMap<R> {
    pub fn new(ranges: Vec<Range>) -> Self {
        let masks = get_masks(ranges.iter()).into_iter().collect();

        Self {
            highest_priority: 0,
            priorities: BTreeSet::new(),
            masks,
            ranges,
            hash_map: HashMap::new(),
        }
    }

    pub fn highest_priority(&self) -> Priority {
        self.highest_priority
    }

    pub fn can_insert(&self, rule: &R) -> bool {
        let rule_ranges = rule.masks().iter().map(|m| {
            if cfg!(debug_assertions) {
                // make sure masks are correctly right-aligned
                let shift = 32 - m.count_ones();
                debug_assert_eq!((m << shift) >> shift, *m);
            }

            // we can simply count the bits to get the prefix length
            m.count_ones()
        });

        self.ranges
            .iter()
            .zip(rule_ranges)
            .all(|((r_low, r_high), r_rule)| r_rule >= *r_low && r_rule < *r_high)
    }

    pub fn insert(&mut self, rule: R) -> bool {
        if !self.priorities.insert(rule.priority()) {
            // We enforce unique priorities
            return false;
        }

        if rule.priority() > self.highest_priority {
            self.highest_priority = rule.priority();
        }

        let hash = self.calc_hash(rule.fields().iter());
        if let Some(rule_list) = self.hash_map.get_mut(&hash) {
            rule_list.push(rule);
        } else {
            self.hash_map.insert(hash, vec![rule]);
        }

        true
    }

    pub fn remove(&mut self, rule: &R) -> bool {
        if !self.priorities.remove(&rule.priority()) {
            return false;
        }

        if rule.priority() == self.highest_priority {
            self.highest_priority = *self.priorities.iter().min().unwrap_or(&0);
        }

        let hash = self.calc_hash(rule.fields().iter());
        // since we added the priority, the rule should be present in the hash_map
        let rule_list = self.hash_map.get_mut(&hash).unwrap();
        let index = rule_list.iter().position(|r| r == rule).unwrap();
        rule_list.swap_remove(index);

        true
    }

    pub fn check_match(&self, packet: &impl Packet) -> Option<&R> {
        let hash = self.calc_hash(packet.fields().iter());

        if let Some(matching_rules) = self.hash_map.get(&hash) {
            let mut best_prio = 0;
            let mut best_match = None;

            for r in matching_rules.iter() {
                if packet
                    .fields()
                    .iter()
                    .zip(r.fields().iter())
                    .zip(r.masks())
                    .all(|((&pf, &rf), &rm)| is_match(pf, rf, rm))
                    && r.priority() > best_prio
                {
                    best_prio = r.priority();
                    best_match = Some(r);
                }
            }

            return best_match;
        }

        None
    }

    fn calc_hash<'a>(&self, fields: impl Iterator<Item = &'a Field>) -> u32 {
        // TODO: this can certainly be improved

        let mut hash = 0;
        let mut p = 1;

        for (m, f) in self.masks.iter().zip(fields) {
            hash ^= p | (f & m);
            p ^= 1;
        }

        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::mocks::{MockPacket, MockRule};

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_is_match() {
        assert!(is_match(0b1101, 0b0101, 0b0111));
        assert!(is_match(0b0101, 0b1101, 0b0111));

        assert!(!is_match(0b1111, 0b1101, 0b0111));
        assert!(!is_match(0b1101, 0b1111, 0b0111));
    }

    #[test]
    fn test_get_mask() {
        let ranges = vec![(3, 5), (6, 10), (1, 2), (0, 1)];

        assert_eq!(get_masks(ranges.iter()), vec![0b111, 0b11_1111, 0b1, 0b0]);
    }

    #[test]
    fn test_rv_hash_map_can_insert_if_can_insert() {
        let map: RVHashMap<MockRule> = RVHashMap::new(vec![(3, 5)]);

        let yes1 = MockRule::new(vec![0b101], vec![0b111], 1);
        let yes2 = MockRule::new(vec![0b101], vec![0b1111], 1);

        assert!(map.can_insert(&yes1));
        assert!(map.can_insert(&yes2));
    }

    #[test]
    fn test_rv_hash_map_can_insert_if_cannot_insert() {
        let map: RVHashMap<MockRule> = RVHashMap::new(vec![(3, 5)]);

        let no1 = MockRule::new(vec![0b101], vec![0b11], 1);
        let no2 = MockRule::new(vec![0b101], vec![0b11111], 1);

        assert!(!map.can_insert(&no1));
        assert!(!map.can_insert(&no2));
    }

    #[test]
    fn test_rv_hash_map_insert_if_priority_is_not_unique() {
        let mut map: RVHashMap<MockRule> = RVHashMap::new(vec![(3, 5)]);

        let yes1 = MockRule::new(vec![0b101], vec![0b111], 1);
        map.insert(yes1);

        let no1 = MockRule::new(vec![0b101], vec![0b111], 1);
        assert!(!map.insert(no1))
    }

    #[test]
    fn test_rv_hash_map_insert_updates_priorities() {
        let mut map: RVHashMap<MockRule> = RVHashMap::new(vec![(3, 5)]);

        let r = MockRule::new(vec![0b101], vec![0b111], 1);
        map.insert(r);
        assert_eq!(map.highest_priority(), 1);

        let r = MockRule::new(vec![0b101], vec![0b1111], 4);
        map.insert(r);
        assert_eq!(map.highest_priority(), 4);

        let r = MockRule::new(vec![0b111], vec![0b111], 2);
        map.insert(r);
        assert_eq!(map.highest_priority(), 4);
    }

    #[test]
    fn test_rv_hash_map_remove_updates_priorities() {
        let mut map: RVHashMap<MockRule> = RVHashMap::new(vec![(3, 5)]);

        let r1 = MockRule::new(vec![0b101], vec![0b111], 1);
        let r2 = MockRule::new(vec![0b11], vec![0b1111], 4);
        let r3 = MockRule::new(vec![0b1001], vec![0b1111], 6);

        map.insert(r1.clone());
        map.insert(r2.clone());
        map.insert(r3.clone());

        map.remove(&r2);
        assert_eq!(map.highest_priority(), 6);

        map.remove(&r3);
        assert_eq!(map.highest_priority(), 1);
    }

    #[test]
    fn test_rv_hash_map_classifies_packets_correctly() {
        let mut map: RVHashMap<MockRule> = RVHashMap::new(vec![(3, 5)]);

        let r1 = MockRule::new(vec![0b101], vec![0b111], 1);
        let r2 = MockRule::new(vec![0b1101], vec![0b1111], 4);
        let r3 = MockRule::new(vec![0b1001], vec![0b1111], 6);

        map.insert(r1);
        map.insert(r2);
        map.insert(r3);

        let p1 = MockPacket::new(vec![0b101]);
        let p2 = MockPacket::new(vec![0b1101]);
        let p3 = MockPacket::new(vec![0b1001]);
        let p4 = MockPacket::new(vec![0b1010]);

        assert_eq!(map.check_match(&p1).expect("should match").priority(), 1);
        assert_eq!(map.check_match(&p2).expect("should match").priority(), 4);
        assert_eq!(map.check_match(&p3).expect("should match").priority(), 6);
        assert!(map.check_match(&p4).is_none());
    }

    #[test]
    fn test_rv_hash_map_check_match_on_multiple_fields() {
        let mut map: RVHashMap<MockRule> = RVHashMap::new(vec![(3, 5), (3, 5)]);
        let r1 = MockRule::new(vec![0b101, 0b1010], vec![0b111, 0b1111], 1);
        map.insert(r1);

        let p1 = MockPacket::new(vec![0b101, 0b1000]);
        let p2 = MockPacket::new(vec![0b100, 0b1010]);
        let p3 = MockPacket::new(vec![0b101, 0b1010]);

        assert!(map.check_match(&p1).is_none());
        assert!(map.check_match(&p2).is_none());

        assert!(map.check_match(&p3).is_some());
    }
}
