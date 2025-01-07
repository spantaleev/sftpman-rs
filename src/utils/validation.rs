use validator::ValidationErrors;

pub fn errors_to_string_list(errors: ValidationErrors) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();

    for (_field_name, field_errors) in errors.field_errors() {
        for field_error in field_errors {
            list.push(field_error.to_string());
        }
    }

    list
}
