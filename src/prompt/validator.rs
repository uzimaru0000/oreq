use std::fmt::Display;

use anyhow::anyhow;
use inquire::validator::{CustomTypeValidator, ErrorMessage, Validation};
use serde_json::Value;

#[derive(Clone)]
pub struct RangeValidator<T: PartialOrd + Clone> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl CustomTypeValidator<String> for RangeValidator<usize> {
    fn validate(&self, input: &String) -> Result<Validation, inquire::CustomUserError> {
        if let Some(min) = self.min {
            if input.len() < min {
                return Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                    "Value must be greater than or equal to {}",
                    min
                ))));
            }
        }

        if let Some(max) = self.max {
            if input.len() > max {
                return Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                    "Value must be less than or equal to {}",
                    max
                ))));
            }
        }

        Ok(Validation::Valid)
    }
}

impl<T: PartialOrd + Clone + Display> CustomTypeValidator<T> for RangeValidator<T> {
    fn validate(&self, input: &T) -> Result<Validation, inquire::CustomUserError> {
        if let Some(min) = self.min.clone() {
            if *input < min {
                return Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                    "Value must be greater than or equal to {}",
                    min
                ))));
            }
        }

        if let Some(max) = self.max.clone() {
            if *input > max {
                return Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                    "Value must be less than or equal to {}",
                    max
                ))));
            }
        }

        Ok(Validation::Valid)
    }
}

impl CustomTypeValidator<Value> for RangeValidator<f64> {
    fn validate(&self, input: &Value) -> Result<Validation, inquire::CustomUserError> {
        match input {
            Value::Null => Err(anyhow!("Value must be a number or string").into()),
            Value::Bool(_) => Err(anyhow!("Value must be a number or string").into()),
            Value::Number(n) => {
                let n = n.as_f64().unwrap();
                let validator = RangeValidator::<f64> {
                    min: self.min,
                    max: self.max,
                };
                validator.validate(&n)
            }
            Value::String(s) => {
                let validator = RangeValidator::<usize> {
                    min: self.min.map(|x| x as usize),
                    max: self.max.map(|x| x as usize),
                };
                validator.validate(s)
            }
            Value::Array(_) => Err(anyhow!("Value must be a number or string").into()),
            Value::Object(_) => Err(anyhow!("Value must be a number or string").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_validator() {
        let validator = RangeValidator::<i64> {
            min: Some(1),
            max: Some(3),
        };

        assert_eq!(
            validator.validate(&1i64).unwrap(),
            Validation::Valid,
            "1 is in range"
        );
        assert_eq!(
            validator.validate(&0i64).unwrap(),
            Validation::Invalid(ErrorMessage::Custom(
                "Value must be greater than or equal to 1".to_owned()
            )),
            "0 is not in range"
        );
        assert_eq!(
            validator.validate(&4i64).unwrap(),
            Validation::Invalid(ErrorMessage::Custom(
                "Value must be less than or equal to 3".to_owned()
            )),
            "4 is not in range"
        );
    }
}
