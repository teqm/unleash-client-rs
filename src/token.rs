use crate::error::TokenParseError;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct UnleashToken {
    pub token: String,
    pub environment: String,
}

impl TryFrom<String> for UnleashToken {
    type Error = TokenParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        UnleashToken::from_str(value.as_str())
    }
}

impl FromStr for UnleashToken {
    type Err = TokenParseError;

    fn from_str(s: &str) -> Result<UnleashToken, Self::Err> {
        if s.contains(':') && s.contains('.') {
            let token_parts: Vec<String> = s.split(':').take(2).map(|s| s.to_string()).collect();

            if let Some(env_and_key) = token_parts.get(1) {
                let env_and_key_parts: Vec<String> = env_and_key
                    .split('.')
                    .take(2)
                    .map(|s| s.to_string())
                    .collect();

                if env_and_key_parts.len() != 2 {
                    return Err(TokenParseError);
                }

                Ok(UnleashToken {
                    environment: env_and_key_parts.get(0).cloned().unwrap(),
                    token: s.to_string(),
                })
            } else {
                Err(TokenParseError)
            }
        } else {
            Err(TokenParseError)
        }
    }
}
