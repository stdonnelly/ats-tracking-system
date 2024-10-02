enum ShellOption {
    /// Initial option state. Should only be used for the startup of the main loop to prevent anything from happening
    Initial,
    /// Any command that is invalid
    Invalid,
    Help,
    Exit,
    Create,
    Read(ReadType),
    /// Has update type and an id
    Update(UpdateType, i32),
    /// Delete `id`
    Delete(i32),
}

enum ReadType {
    All,
    Pending,
    /// Search for specific string
    Search(String),
    /// Show only `id`
    One(i32),
}

enum UpdateType {
    HumanResponse,
    Other
}

impl From<&str> for ShellOption {
    fn from(value: &str) -> Self {
        match value {
            "help" | "h" => Self::Help,
            "exit" | "quit" => Self::Exit,
            "create" => Self::Create,
            // TODO...
            // "read" | "ls" => Self::Read(()),
            _ => Self::Invalid
        }
    }
}

fn main_loop() {

}