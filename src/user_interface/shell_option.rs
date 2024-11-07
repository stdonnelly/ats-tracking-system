#[derive(Debug)]
pub(super) enum ShellOption {
    Help,
    Exit,
    Create,
    Read(ReadType),
    /// Has update type and an id
    Update(UpdateType, i32),
    /// Delete `id`
    Delete(i32),
}

#[derive(Debug)]
pub(super) enum ReadType {
    All,
    Pending,
    /// Search for specific string
    Search(String),
    /// Show only `id`
    One(i32),
}

#[derive(Debug)]
pub(super) enum UpdateType {
    HumanResponse,
    Other,
}

impl TryFrom<&str> for ShellOption {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Guard clause to make sure some command was given
        if value == "" {
            return Err("No command given".to_owned());
        }

        // Determine command and args (if any)
        let command_word: &str;
        let args: &str;
        if let Some(command_args) = value.split_once(' ') {
            (command_word, args) = command_args;
        } else {
            // Set command word and just leave args blank
            command_word = value;
            args = "";
        }

        match command_word {
            "help" | "h" => Ok(Self::Help),
            "exit" | "quit" => Ok(Self::Exit),
            "create" | "new" => Ok(Self::Create),
            // For read command, parse the read type as well
            "read" => Ok(Self::Read(ReadType::try_from(args)?)),
            // Searching, because it is so common, can just used the command "search" instead
            // Pass command and args in this case so try_from gets the string "search" as well
            "search" => Ok(Self::Read(ReadType::try_from(value)?)),
            // For update command, parse the read type and the id
            "update" => {
                if let Some((update_type_str, id_str)) = args.split_once(' ') {
                    Ok(Self::Update(
                        // Parse update type
                        UpdateType::try_from(update_type_str)?,
                        // Parse id
                        str::parse::<i32>(id_str).map_err(|err_message| {
                            format!("Unable to parse id '{id_str}'. Error: {err_message}")
                        })?,
                    ))
                } else {
                    Err("Update type and ID are required".to_owned())
                }
            }
            // For delete, parse the id
            "delete" => match str::parse::<i32>(args) {
                Ok(id) => Ok(Self::Delete(id)),
                Err(err_message) => {
                    Err(format!("Unable to parse id '{args}'. Error: {err_message}"))
                }
            },
            _ => Err("Invalid command".to_owned()),
        }
    }
}

impl TryFrom<&str> for ReadType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Guard clause intentionally omitted because a naked `read` should act like `read all`

        // Determine command and args (if any)
        let command_word: &str;
        let args: &str;
        if let Some(command_args) = value.split_once(' ') {
            (command_word, args) = command_args;
        } else {
            // Set command word and just leave args blank
            command_word = value;
            args = "";
        }

        match command_word {
            "all" | "" => Ok(Self::All),
            "pending" => Ok(Self::Pending),
            "search" => {
                if args != "" {
                    Ok(Self::Search(args.to_owned()))
                } else {
                    Err("Search query is required for search".to_owned())
                }
            }
            _ => match str::parse::<i32>(command_word) {
                Ok(id) => Ok(Self::One(id)),
                Err(err_message) => Err(format!(
                    "Unable to parse id '{command_word}'. Error: {err_message}"
                )),
            },
        }
    }
}

impl TryFrom<&str> for UpdateType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "response" => Ok(Self::HumanResponse),
            "other" => Ok(Self::Other),
            "" => Err("No update type given".to_owned()),
            _ => Err("Invalid update type".to_owned()),
        }
    }
}
