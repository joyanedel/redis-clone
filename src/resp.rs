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

impl TryFrom<&[&str]> for RESPValues {
    type Error = ();

    fn try_from(value: &[&str]) -> Result<Self, Self::Error> {
        if value.len() == 0 {
            todo!("Returns error if len of value is 0");
        }

        let mut value = value.iter();
        let first_element = value.next().unwrap();

        // Match all single line elements
        // Match simple strings
        if let Some(v) = Regex::new(r"^\+(?<value>.+)$")
            .unwrap()
            .captures(&first_element)
        {
            return Ok(Self::SimpleString(v["value"].to_string()));
        }
        // Match simple errors
        if let Some(v) = Regex::new("^-(?<value>.+)$")
            .unwrap()
            .captures(&first_element)
        {
            return Ok(Self::SimpleError(v["value"].to_string()));
        }
        // Match 64bit integers
        if let Some(v) = Regex::new(r"^:(?<value>(\+|-)?\d+)$")
            .unwrap()
            .captures(&first_element)
        {
            return match &v["value"].parse::<i64>() {
                Ok(v) => Ok(Self::Integer(*v)),
                Err(_) => todo!("Resolve Error in integer match"),
            };
        }

        // Match all 2+ lines elements
        Ok(Self::BigNumber)
    }
}

#[cfg(test)]
mod tests {
    use super::RESPValues;

    #[test]
    fn parse_simple_string_correctly() {
        let value = vec!["+PING"];
        let result = RESPValues::try_from(value.as_ref());

        assert!(result.is_ok_and(|r| r == RESPValues::SimpleString("PING".to_string())))
    }

    #[test]
    fn parse_simple_error_correctly() {
        let value = vec!["-TEST ERROR"];
        let result = RESPValues::try_from(value.as_ref());

        assert!(result.is_ok_and(|r| r == RESPValues::SimpleError("TEST ERROR".to_string())));
    }

    #[test]
    fn parse_integer_correctly() {
        let value = vec![":2"];
        let result = RESPValues::try_from(value.as_ref());

        assert!(result.is_ok_and(|r| r == RESPValues::Integer(2)));
    }

    #[test]
    fn parse_negative_integer_correctly() {
        let value = vec![":-2"];
        let result = RESPValues::try_from(value.as_ref());

        assert!(result.is_ok_and(|r| r == RESPValues::Integer(-2)));
    }

    #[test]
    fn parse_bulk_string_correctly() {
        let value = vec!["$Bulk"];
        let result = RESPValues::try_from(value.as_ref());

        assert!(result.is_ok_and(|r| r == RESPValues::BulkString("Bulk".to_string())));
    }

    #[test]
    fn parse_array_with_zero_items_correctly() {
        let value = vec!["*0"];
        let result = RESPValues::try_from(value.as_ref());

        assert!(result.is_ok_and(|r| r == RESPValues::Array(vec![])));
    }

    #[test]
    fn parse_array_with_one_item_correctly() {
        let value = vec!["*1", ":1"];
        let result = RESPValues::try_from(value.as_ref());

        assert!(result.is_ok_and(|r| r == RESPValues::Array(vec![RESPValues::Integer(1)])));
    }
}
