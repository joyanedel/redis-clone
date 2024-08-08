#[derive(PartialEq)]
enum RESPValues {
    // RESP2
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RESPValues>),
    // RESP3
    Null,
    Boolean,
    Double,
    BigNumber,
    BulkError,
    VerbatimString,
    Map,
    Set,
    Push,
}

impl TryFrom<&str> for RESPValues {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self::Push)
    }
}

#[cfg(test)]
mod tests {
    use super::RESPValues;

    #[test]
    fn parse_simple_string_correctly() {
        let raw_string = "+PING\r\n";
        let result = RESPValues::try_from(raw_string);

        assert!(result.is_ok_and(|r| r == RESPValues::SimpleString("PING".to_string())))
    }

    #[test]
    fn parse_simple_error_correctly() {
        let raw_string = "-TEST ERROR\r\n";
        let result = RESPValues::try_from(raw_string);

        assert!(result.is_ok_and(|r| r == RESPValues::SimpleError("TEST ERROR".to_string())));
    }

    #[test]
    fn parse_integer_correctly() {
        let raw_string = ":2\r\n";
        let result = RESPValues::try_from(raw_string);

        assert!(result.is_ok_and(|r| r == RESPValues::Integer(2)));
    }

    #[test]
    fn parse_bulk_string_correctly() {
        let raw_string = "$Bulk\r\n";
        let result = RESPValues::try_from(raw_string);

        assert!(result.is_ok_and(|r| r == RESPValues::BulkString("Bulk".to_string())));
    }

    #[test]
    fn parse_array_with_zero_items_correctly() {
        let raw_string = "*0\r\n";
        let result = RESPValues::try_from(raw_string);

        assert!(result.is_ok_and(|r| r == RESPValues::Array(vec![])));
    }

    #[test]
    fn parse_array_with_one_item_correctly() {
        let raw_string = "*1\r\n:1\r\n";
        let result = RESPValues::try_from(raw_string);

        assert!(result.is_ok_and(|r| r == RESPValues::Array(vec![RESPValues::Integer(1)])));
    }
}
