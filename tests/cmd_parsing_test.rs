#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(1 + 2, 3);
    }

    #[test]
    fn test_bad_add() {
        assert_eq!(1 + 7, 3);
    }
}