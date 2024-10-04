use std::str::Split;

enum ShellOption {
    /// Initial option state. Should only be used for the startup of the main loop to prevent anything from happening
    Initial,
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
    Other,
}

impl TryFrom<&str> for ShellOption {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut value_words = value.split(' ');
        if let Some(command_word) = value_words.next() {
            match command_word {
                "help" | "h" => Ok(Self::Help),
                "exit" | "quit" => Ok(Self::Exit),
                "create" => Ok(Self::Create),
                // For read command, parse the read type as well
                "read" => Ok(Self::Read(ReadType::try_from(&mut value_words)?)),
                // For update command, parse the read type and the id
                "update" => {
                    let update_type = UpdateType::try_from(&mut value_words)?;
                    if let Some(id_str) = value_words.next() {
                        match str::parse::<i32>(id_str) {
                            Ok(id) => Ok(Self::Update(update_type, id)),
                            Err(err_message) => Err(format!(
                                "Unable to parse id '{id_str}'. Error: {err_message}"
                            )),
                        }
                    } else {
                        Err("ID is required for update".to_owned())
                    }
                }
                // For delete, parse the id
                "delete" => {
                    if let Some(id_str) = value_words.next() {
                        match str::parse::<i32>(id_str) {
                            Ok(id) => Ok(Self::Delete(id)),
                            Err(err_message) => Err(format!(
                                "Unable to parse id '{id_str}'. Error: {err_message}"
                            )),
                        }
                    } else {
                        Err("ID is required for delete".to_owned())
                    }
                }
                _ => Err("Invalid command".to_owned()),
            }
        } else {
            Err("No command given".to_owned())
        }
    }
}

impl TryFrom<&mut Split<'_, char>> for ReadType {
    type Error = String;

    fn try_from(value: &mut Split<'_, char>) -> Result<Self, Self::Error> {
        if let Some(read_type) = value.next() {
            match read_type {
                "all" => Ok(Self::All),
                "pending" => Ok(Self::Pending),
                "search" => {
                    if let Some(search_query) = value.next() {
                        Ok(Self::Search(search_query.to_owned()))
                    } else {
                        Err("Search query is required for search".to_owned())
                    }
                }
                "one" => {
                    if let Some(id_str) = value.next() {
                        match str::parse::<i32>(id_str) {
                            Ok(id) => Ok(Self::One(id)),
                            Err(err_message) => Err(format!(
                                "Unable to parse id '{id_str}'. Error: {err_message}"
                            )),
                        }
                    } else {
                        Err("ID is required for read one".to_owned())
                    }
                }
                _ => Err("Invalid read type".to_owned()),
            }
        } else {
            Err("No read type given".to_owned())
        }
    }
}

impl TryFrom<&mut Split<'_, char>> for UpdateType {
    type Error = String;

    fn try_from(value: &mut Split<'_, char>) -> Result<Self, Self::Error> {
        if let Some(update_type) = value.next() {
            match update_type {
                "response" => Ok(Self::HumanResponse),
                "other" => Ok(Self::Other),
                _ => Err("Invalid update type".to_owned()),
            }
        } else {
            Err("No update type given".to_owned())
        }
    }
}

fn main_loop() {}
