// Test fixtures and helpers

/// Example test fixture for setting up test state
pub struct TestFixture {
    pub name: String,
}

impl TestFixture {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creation() {
        let fixture = TestFixture::new("example");
        assert_eq!(fixture.name, "example");
    }
}
