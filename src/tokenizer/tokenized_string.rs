use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::tokenizer::{Token, TokenStream};

/// Struct representing pre-tokenized text
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PreTokenizedString {
    /// Original text
    pub text: String,
    /// Tokens derived from the text
    pub tokens: Vec<Token>,
}

impl Ord for PreTokenizedString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.text.cmp(&other.text)
    }
}

impl PartialOrd for PreTokenizedString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// TokenStream implementation which wraps [PreTokenizedString]
pub struct PreTokenizedStream {
    tokenized_string: PreTokenizedString,
    current_token: i64,
}

impl From<PreTokenizedString> for PreTokenizedStream {
    fn from(s: PreTokenizedString) -> PreTokenizedStream {
        PreTokenizedStream {
            tokenized_string: s,
            current_token: -1,
        }
    }
}

impl TokenStream for PreTokenizedStream {
    fn advance(&mut self) -> bool {
        self.current_token += 1;
        self.current_token < self.tokenized_string.tokens.len() as i64
    }

    fn token(&self) -> &Token {
        assert!(
            self.current_token >= 0,
            "TokenStream not initialized. You should call advance() at least once."
        );
        &self.tokenized_string.tokens[self.current_token as usize]
    }

    fn token_mut(&mut self) -> &mut Token {
        assert!(
            self.current_token >= 0,
            "TokenStream not initialized. You should call advance() at least once."
        );
        &mut self.tokenized_string.tokens[self.current_token as usize]
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tokenizer::Token;

    #[test]
    fn test_tokenized_stream() {
        let tok_text = PreTokenizedString {
            text: String::from("A a"),
            tokens: vec![
                Token {
                    offset_from: 0,
                    offset_to: 1,
                    position: 0,
                    text: String::from("A"),
                    position_length: 1,
                },
                Token {
                    offset_from: 2,
                    offset_to: 3,
                    position: 1,
                    text: String::from("a"),
                    position_length: 1,
                },
            ],
        };

        let mut token_stream = PreTokenizedStream::from(tok_text.clone());

        for expected_token in tok_text.tokens {
            assert!(token_stream.advance());
            assert_eq!(token_stream.token(), &expected_token);
        }
        assert!(!token_stream.advance());
    }
}
