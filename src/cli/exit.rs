pub enum Status {
    Success,
    DefinitionAlreadyExists,
    DefinitionNotFound,
    Failure,
    UnknownCommand,
    ValidationFailure,
}

impl From<Status> for i32 {
    fn from(value: Status) -> i32 {
        match value {
            Status::Success => 0,
            Status::DefinitionAlreadyExists => 10,
            Status::DefinitionNotFound => 11,
            Status::Failure => 20,
            Status::UnknownCommand => 30,
            Status::ValidationFailure => 40,
        }
    }
}
