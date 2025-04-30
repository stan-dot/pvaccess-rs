
#[derive(Debug)]
pub enum ClientCommand {
    SendEcho(Vec<u8>),
    Shutdown,
    // Add more commands later (e.g. CreateChannel, Monitor, etc)
}

