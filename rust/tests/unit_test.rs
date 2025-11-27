// Unit tests for individual modules

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder_unit_test() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_string_operations() {
        let s = "hello".to_string();
        assert_eq!(s.len(), 5);
        assert!(s.starts_with("hel"));
    }
}
