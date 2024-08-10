use crate::resp::RESPValues;

#[derive(PartialEq, Debug)]
pub enum RedisCommand {
    Ping(Option<String>),
    Echo(String),
    CommandDocs(Option<String>),
}

impl TryFrom<RESPValues> for RedisCommand {
    type Error = ();
    fn try_from(value: RESPValues) -> Result<Self, Self::Error> {
        let array = match value {
            RESPValues::Array(v) if !v.is_empty() => v,
            _ => todo!("Handle value not being array variant of RESPValues"),
        };

        // match command docs
        if array[0] == RESPValues::BulkString("COMMAND".to_string())
            && array[1] == RESPValues::BulkString("DOCS".to_string())
        {
            let sub_command = array.get(2);
            return Ok(Self::CommandDocs(sub_command.and_then(|v| match v {
                RESPValues::BulkString(s) => Some(s.clone()),
                _ => None,
            })));
        }

        // match ping
        if array[0] == RESPValues::BulkString("PING".to_string()) {
            let echoed_string = array.get(1).and_then(|v| match v {
                RESPValues::BulkString(s) => Some(s.to_string()),
                _ => None,
            });
            return Ok(Self::Ping(echoed_string));
        }

        // match echo
        if array[0] == RESPValues::BulkString("ECHO".to_string()) {
            let echoed_string = match array.get(1) {
                Some(RESPValues::BulkString(v)) => v.to_owned(),
                _ => todo!("raise an error if echoed string is absent in echo command"),
            };
            return Ok(RedisCommand::Echo(echoed_string));
        }

        unimplemented!()
    }
}

#[cfg(test)]
mod command_tests {
    use crate::{commands::RedisCommand, resp::RESPValues};

    #[test]
    fn parse_command_docs_with_no_string_correctly() {
        let value = RESPValues::Array(vec![
            RESPValues::BulkString("COMMAND".to_string()),
            RESPValues::BulkString("DOCS".to_string()),
        ]);
        let result = RedisCommand::try_from(value);

        assert!(result.is_ok_and(|r| r == RedisCommand::CommandDocs(None)));
    }

    #[test]
    fn parse_command_docs_with_a_string_correctly() {
        let value = RESPValues::Array(vec![
            RESPValues::BulkString("COMMAND".to_string()),
            RESPValues::BulkString("DOCS".to_string()),
            RESPValues::BulkString("SET".to_string()),
        ]);
        let result = RedisCommand::try_from(value);

        assert!(result.is_ok_and(|r| r == RedisCommand::CommandDocs(Some("SET".to_string()))));
    }

    #[test]
    fn parse_ping_with_no_string_correctly() {
        let value = RESPValues::Array(vec![RESPValues::BulkString("PING".to_string())]);
        let result = RedisCommand::try_from(value);

        assert!(result.is_ok_and(|r| r == RedisCommand::Ping(None)));
    }

    #[test]
    fn parse_ping_with_one_string_correctly() {
        let value = RESPValues::Array(vec![
            RESPValues::BulkString("PING".to_string()),
            RESPValues::BulkString("testing".to_string()),
        ]);
        let result = RedisCommand::try_from(value);

        assert!(result.is_ok_and(|r| r == RedisCommand::Ping(Some("testing".to_string()))));
    }

    #[test]
    fn parse_echo_with_string_correctly() {
        let value = RESPValues::Array(vec![
            RESPValues::BulkString("ECHO".to_string()),
            RESPValues::BulkString("testing".to_string()),
        ]);
        let result = RedisCommand::try_from(value);

        assert!(result.is_ok_and(|r| r == RedisCommand::Echo("testing".to_string())));
    }
}
