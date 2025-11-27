// Integration tests for ephemeral-rollups-sdk

mod common {
    pub mod fixtures;
    pub use fixtures::TestFixture;
}

#[cfg(test)]
mod tests {
    use crate::common::TestFixture;

    #[test]
    fn test_basic_fixture() {
        let fixture = TestFixture::new("integration_test");
        assert!(!fixture.name.is_empty());
    }

    #[test]
    fn test_multiple_fixtures() {
        let fixture1 = TestFixture::new("test1");
        let fixture2 = TestFixture::new("test2");

        assert_ne!(fixture1.name, fixture2.name);
    }
}
