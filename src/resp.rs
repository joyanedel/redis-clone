use regex::Regex;

#[derive(PartialEq, Debug)]
pub enum RESPValues {
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
            let mut remaining_elements = rest_elements.to_string();

            for _ in 0..n {
                let result = match RESPValues::try_from(remaining_elements.as_str()) {
                    Ok(v) => v,
                    Err(_) => todo!("Handle recursive array try from"),
                };

                remaining_elements = remaining_elements.replacen(&result.to_string(), "", 1);
                array.push(result);
            }

            return Ok(Self::Array(array));
        }

        Ok(Self::BigNumber)
    }
}

impl ToString for RESPValues {
    fn to_string(&self) -> String {
        match self {
            Self::SimpleString(v) => format!("+{v}\r\n"),
            Self::SimpleError(v) => format!("-{v}\r\n"),
            Self::Integer(v) => format!(":{v}\r\n"),
            Self::BulkString(v) => format!("${}\r\n{}\r\n", v.len(), v),
            Self::Array(v) => {
                let length = v.len();
                let elements_repr: Vec<_> = v.iter().map(|e| e.to_string()).collect();
                let elements_repr = elements_repr.join("");
                format!("*{length}\r\n{elements_repr}")
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod impl_try_from_for_resp {
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

#[cfg(test)]
mod impl_to_string_for_resp {
    use super::RESPValues;

    #[test]
    fn simple_string_to_string() {
        let value = RESPValues::SimpleString(String::from("PING"));
        let result = value.to_string();
        assert_eq!(&result, "+PING\r\n");
    }

    #[test]
    fn simple_error_to_string() {
        let value = RESPValues::SimpleError(String::from("TEST ERROR"));
        let result = value.to_string();
        assert_eq!(&result, "-TEST ERROR\r\n");
    }

    #[test]
    fn integer_to_string() {
        let value = RESPValues::Integer(10);
        let result = value.to_string();
        assert_eq!(&result, ":10\r\n");
    }

    #[test]
    fn negative_integer_to_string() {
        let value = RESPValues::Integer(-10);
        let result = value.to_string();
        assert_eq!(&result, ":-10\r\n");
    }

    #[test]
    fn bulk_string_to_string() {
        let value = RESPValues::BulkString(String::from("testing"));
        let result = value.to_string();
        assert_eq!(&result, "$7\r\ntesting\r\n");
    }

    #[test]
    fn empty_array_to_string() {
        let value = RESPValues::Array(vec![]);
        let result = value.to_string();
        assert_eq!(&result, "*0\r\n");
    }

    #[test]
    fn one_item_array_to_string() {
        let value = RESPValues::Array(vec![RESPValues::Integer(2)]);
        let result = value.to_string();
        assert_eq!(&result, "*1\r\n:2\r\n");
    }

    #[test]
    fn nested_items_array_to_string() {
        let value = RESPValues::Array(vec![
            RESPValues::Integer(2),
            RESPValues::Array(vec![RESPValues::BulkString(String::from("PONG"))]),
        ]);
        let result = value.to_string();
        assert_eq!(&result, "*2\r\n:2\r\n*1\r\n$4\r\nPONG\r\n");
    }
}
