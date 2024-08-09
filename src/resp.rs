use regex::Regex;

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
        if value.len() == 0 {
            todo!("Returns error if len of value is 0");
        }

        let (first_element, rest_elements) = match value.split_once("\r\n") {
            Some((first, rest)) => (first, rest),
            None => todo!("Handle split once \\r\\n"),
        };

        // Match all single line elements
        // Match simple strings
        if let Some(captures) = Regex::new(r"^\+(?<value>.+)$")
            .unwrap()
            .captures(&first_element)
        {
            return Ok(Self::SimpleString(captures["value"].to_string()));
        }
        // Match simple errors
        if let Some(captures) = Regex::new("^-(?<value>.+)$")
            .unwrap()
            .captures(&first_element)
        {
            return Ok(Self::SimpleError(captures["value"].to_string()));
        }
        // Match 64bit integers
        if let Some(captures) = Regex::new(r"^:(?<value>(\+|-)?\d+)$")
            .unwrap()
            .captures(&first_element)
        {
            return match &captures["value"].parse::<i64>() {
                Ok(v) => Ok(Self::Integer(*v)),
                Err(_) => todo!("Resolve Error in integer match"),
            };
        }

        // Match all 2+ lines elements
        // Match bulk string
        if let Some(_) = Regex::new(r"^\$\d+").unwrap().captures(&first_element) {
            return match rest_elements.split("\r\n").next() {
                None => todo!("Handle none in match bulk string"),
                Some(v) => Ok(Self::BulkString(v.to_string())),
            };
        }

        // Match arrays
        if let Some(captures) = Regex::new(r"^\*(?<array_length>\d+)$")
            .unwrap()
            .captures(&first_element)
        {
            let n = match captures["array_length"].parse::<usize>() {
                Ok(v) => v,
                Err(_) => todo!("Array size not usize parseable"),
            };
            let mut array = Vec::with_capacity(n);
            return Ok(Self::Array(array));
        }

        Ok(Self::BigNumber)
    }
}

#[cfg(test)]
mod tests {
    use super::RESPValues;

    #[test]
    fn parse_simple_string_correctly() {
        let value = "+PING\r\n";
        let result = RESPValues::try_from(value);

        assert!(result.is_ok_and(|r| r == RESPValues::SimpleString("PING".to_string())))
    }

    #[test]
    fn parse_simple_error_correctly() {
        let value = "-TEST ERROR\r\n";
        let result = RESPValues::try_from(value);

        assert!(result.is_ok_and(|r| r == RESPValues::SimpleError("TEST ERROR".to_string())));
    }

    #[test]
    fn parse_integer_correctly() {
        let value = ":2\r\n";
        let result = RESPValues::try_from(value);

        assert!(result.is_ok_and(|r| r == RESPValues::Integer(2)));
    }

    #[test]
    fn parse_negative_integer_correctly() {
        let value = ":-2\r\n";
        let result = RESPValues::try_from(value);

        assert!(result.is_ok_and(|r| r == RESPValues::Integer(-2)));
    }

    #[test]
    fn parse_bulk_string_correctly() {
        let value = "$4\r\nBulk\r\n";
        let result = RESPValues::try_from(value);

        assert!(result.is_ok_and(|r| r == RESPValues::BulkString("Bulk".to_string())));
    }

    #[test]
    fn parse_empty_bulk_string_correctly() {
        let value = "$0\r\n\r\n";
        let result = RESPValues::try_from(value);

        assert!(result.is_ok_and(|r| r == RESPValues::BulkString(String::new())));
    }

    #[test]
    fn parse_array_with_zero_items_correctly() {
        let value = "*0\r\n";
        let result = RESPValues::try_from(value);

        assert!(result.is_ok_and(|r| r == RESPValues::Array(vec![])));
    }

    #[test]
    fn parse_array_with_one_item_correctly() {
        let value = "*1\r\n:1\r\n";
        let result = RESPValues::try_from(value);

        assert!(result.is_ok_and(|r| r == RESPValues::Array(vec![RESPValues::Integer(1)])));
    }

    #[test]
    fn parse_nested_array_correctly() {
        let value = "*2\r\n*1\r\n+PING\r\n$4\r\nPONG\r\n";
        let result = RESPValues::try_from(value);

        assert!(result.is_ok_and(|r| r
            == RESPValues::Array(vec![
                RESPValues::Array(vec![RESPValues::SimpleString("PING".to_string())]),
                RESPValues::BulkString("PONG".to_string())
            ])));
    }
}
