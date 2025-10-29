use crate::map_env::Placeholder;
use anyhow::Context;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct PlaceholderContent {
    pub key: String,
    pub default_value: Option<String>,
}

impl PlaceholderContent {
    pub fn parse(content: &str) -> Self {
        if let Some(colon_pos) = content.find(':') {
            let key = content[..colon_pos].trim().to_string();
            let default = content[colon_pos + 1..].trim().to_string();

            Self {
                key,
                default_value: if default.is_empty() {
                    None
                } else {
                    Some(default)
                },
            }
        } else {
            Self {
                key: content.trim().to_string(),
                default_value: None,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaceholderMatch {
    pub full_text: String,
    pub start: usize,
    pub end: usize,
    pub content: PlaceholderContent,
    pub style: Placeholder,
}

#[derive(Debug, Clone)]
pub enum ReplacementValue {
    EnvValue(String),
    DefaultValue(String),
    NoValue,
}

pub struct PlaceholderParser {
    patterns: HashMap<Placeholder, regex::Regex>,
}

impl PlaceholderParser {
    pub fn new() -> anyhow::Result<Self> {
        let mut patterns = HashMap::new();

        for style in [
            Placeholder::Underscore,
            Placeholder::DoubleCurly,
            Placeholder::DollarCurly,
            Placeholder::DollarBrace,
        ] {
            let pattern = regex::Regex::new(style.regex_pattern())
                .context(format!("Failed to compile regex for {:?}", style))?;
            patterns.insert(style, pattern);
        }

        Ok(Self { patterns })
    }

    /// Find all placeholder matches in text for a specific style
    pub fn find_matches(&self, text: &str, style: &Placeholder) -> Vec<PlaceholderMatch> {
        let pattern = self
            .patterns
            .get(style)
            .expect("Pattern should exist for all styles");

        let mut matches = Vec::new();

        for cap in pattern.captures_iter(text) {
            let full_match = cap.get(0).unwrap();
            let content_str = cap.get(1).unwrap().as_str();

            // Parse content to extract key and default
            let content = PlaceholderContent::parse(content_str);

            matches.push(PlaceholderMatch {
                full_text: full_match.as_str().to_string(),
                start: full_match.start(),
                end: full_match.end(),
                content,
                style: style.clone(),
            });
        }

        matches
    }
}

pub struct Replacer {
    parser: PlaceholderParser,
}

impl Replacer {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            parser: PlaceholderParser::new()?,
        })
    }

    fn resolve_value(
        &self,
        placeholder: &PlaceholderMatch,
        env_values: &HashMap<String, String>,
    ) -> ReplacementValue {
        let key = &placeholder.content.key;

        if let Some(env_val) = env_values.get(key)
            && !placeholder.full_text.eq_ignore_ascii_case(env_val)
        {
            if !env_val.is_empty() {
                return ReplacementValue::EnvValue(env_val.clone());
            }
        }

        if let Some(default) = &placeholder.content.default_value {
            return ReplacementValue::DefaultValue(default.clone());
        }

        ReplacementValue::NoValue
    }

    pub fn replace_all(
        &self,
        text: &str,
        env_values: &HashMap<String, String>,
        placeholder_style: &Placeholder,
    ) -> anyhow::Result<String> {
        let matches = self.parser.find_matches(text, placeholder_style);

        if matches.is_empty() {
            return Ok(text.to_string());
        }

        let mut result = text.to_string();

        for placeholder in matches.iter().rev() {
            let replacement_value = self.resolve_value(placeholder, env_values);

            let replacement_text = match replacement_value {
                ReplacementValue::EnvValue(val) => val,
                ReplacementValue::DefaultValue(val) => val,
                ReplacementValue::NoValue => placeholder.full_text.clone(),
            };

            // Replace in result string
            result.replace_range(placeholder.start..placeholder.end, &replacement_text);
        }

        Ok(result)
    }
}
