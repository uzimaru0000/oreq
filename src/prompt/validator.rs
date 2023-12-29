use inquire::validator::{CustomTypeValidator, ErrorMessage, StringValidator, Validation};

#[derive(Clone)]
pub struct RangeValidator<T: PartialOrd + Clone> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl StringValidator for RangeValidator<usize> {
    fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
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

impl CustomTypeValidator<f64> for RangeValidator<f64> {
    fn validate(&self, input: &f64) -> Result<Validation, inquire::CustomUserError> {
        if let Some(min) = self.min {
            if *input < min as f64 {
                return Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                    "Value must be greater than or equal to {}",
                    min
                ))));
            }
        }

        if let Some(max) = self.max {
            if *input > max as f64 {
                return Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                    "Value must be less than or equal to {}",
                    max
                ))));
            }
        }

        Ok(Validation::Valid)
    }
}

impl CustomTypeValidator<i64> for RangeValidator<i64> {
    fn validate(&self, input: &i64) -> Result<Validation, inquire::CustomUserError> {
        if let Some(min) = self.min {
            if *input < min as i64 {
                return Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                    "Value must be greater than or equal to {}",
                    min
                ))));
            }
        }

        if let Some(max) = self.max {
            if *input > max as i64 {
                return Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                    "Value must be less than or equal to {}",
                    max
                ))));
            }
        }

        Ok(Validation::Valid)
    }
}
