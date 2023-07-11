pub mod color;
pub mod emote;
pub mod message;

pub use crate::common::emote::EmoteServer;
pub use crate::common::message::IrcMessage;

// Twitch IRC secure web socket URL
pub const SOCKET_URL: &str = "wss://irc-ws.chat.twitch.tv:443";
