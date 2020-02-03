mod classifier;
mod range_vector_hash_map;
pub mod types;

pub mod prelude {
    pub use super::classifier::RVHClassifier;
    pub use super::types::*;
}

pub use classifier::RVHClassifier;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
