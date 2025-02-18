use pest::{
    error::{Error, ErrorVariant},
    Parser,
};
use pest_derive::Parser;

use super::error::FilterError;

#[derive(Parser)]
#[grammar = "filter/grammar.pest"]
pub struct Filter {}

/// one segment of a filter ([`parse_filter`] returns `Vec<Field>`)
///
/// e.g. for the filter `'.foo{"bar"}.baz'` there are two segments:
///
/// * the name "foo" and the label "bar"
/// * the name "baz"
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    /// an attribute, block, or object name
    pub name: String,
    /// block labels or object keys
    pub labels: Vec<String>,
    /// a list index
    pub index: Option<usize>,
}

impl Field {
    pub fn new(name: &str) -> Self {
        Field {
            name: name.to_string(),
            labels: Vec::new(),
            index: None,
        }
    }

    pub fn labeled(name: &str, labels: &[&str]) -> Self {
        Field {
            name: name.to_string(),
            labels: labels.iter().map(|label| label.to_string()).collect(),
            index: None,
        }
    }

    pub fn indexed(name: &str, index: usize) -> Self {
        Field {
            name: name.to_string(),
            labels: Vec::new(),
            index: Some(index),
        }
    }
}

/// parse `input` and return a vector of [`Field`]s
///
/// a valid filter is one or more chained segments
pub fn parse_filter(input: &str) -> Result<Vec<Field>, Box<FilterError<Rule>>> {
    let mut fields = Vec::new();
    let pairs = Filter::parse(Rule::filter, input)?;
    for pair in pairs {
        let mut name = String::new();
        let mut labels = Vec::new();
        let mut index: Option<usize> = None;

        let inner_pairs = pair.into_inner();
        for inner in inner_pairs {
            match inner.as_rule() {
                Rule::name => {
                    // according to clippy, this is a more efficient way of doing
                    // `name = inner.as_str().to_owned()`
                    inner.as_str().clone_into(&mut name);
                }
                Rule::quoted_name => {
                    inner.as_str().clone_into(&mut name);
                }
                Rule::label => {
                    labels.push(inner.as_str().to_owned());
                }
                Rule::numeric_index => {
                    if let Ok(i) = inner.as_str().parse::<usize>() {
                        index = Some(i);
                    } else {
                        return Err(Error::new_from_span(
                            ErrorVariant::ParsingError {
                                positives: Vec::new(),
                                negatives: vec![inner.as_rule()],
                            },
                            inner.as_span(),
                        )
                        .into());
                    }
                }
                _ => {
                    return Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: "not implemented".to_string(),
                        },
                        inner.as_span(),
                    )
                    .into())
                }
            }
        }

        if !name.is_empty() {
            fields.push(Field {
                name,
                labels,
                index,
            });
        }
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_filter() {
        let input = ".a_name";
        let expected = vec![Field::new("a_name")];
        let fields = parse_filter(input).expect("failed to parse");
        assert_eq!(expected, fields);
    }

    #[test]
    fn label_filter() {
        let input = ".a_name{\"a_label\"}";
        let expected = vec![Field::labeled("a_name", &["a_label"])];
        let fields = parse_filter(input).expect("failed to parse");
        assert_eq!(expected, fields);
    }

    #[test]
    fn index_filter() {
        let input = ".a_name[0]";
        let expected = vec![Field::indexed("a_name", 0)];
        let fields = parse_filter(input).expect("failed to parse");
        assert_eq!(expected, fields);
    }

    #[test]
    fn traversal_filter() {
        let input = ".a_name{\"a_label\"}.another_name{\"another_label\"}.third_name";
        let expected = vec![
            Field::labeled("a_name", &["a_label"]),
            Field::labeled("another_name", &["another_label"]),
            Field::new("third_name"),
        ];
        let fields = parse_filter(input).expect("failed to parse");
        assert_eq!(expected, fields);
    }

    #[test]
    fn name_starting_with_digit_should_err() {
        let input = ".00asdf";
        parse_filter(input).expect_err("succesfully parsed invalid name");
    }
}
