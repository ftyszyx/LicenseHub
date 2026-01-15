use crate::core::my_error::AppError;

// 验证辅助函数
pub fn validate_not_empty(field: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::validation(format!("{} cannot be empty", field)));
    }
    Ok(())
}

pub fn validate_min_length(field: &str, value: &str, min: usize) -> Result<(), AppError> {
    if value.len() < min {
        return Err(AppError::validation(format!(
            "{} must be at least {} characters",
            field, min
        )));
    }
    Ok(())
}

pub fn validate_positive(field: &str, value: i32) -> Result<(), AppError> {
    if value <= 0 {
        return Err(AppError::validation(format!("{} must be positive", field)));
    }
    Ok(())
}
