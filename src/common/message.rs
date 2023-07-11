use std::{
    fmt::Display,
    io::{Error, ErrorKind},
};

use crate::common::color::get_user_color;
use crate::common::emote::{Emote, EmoteServer};

#[derive(Debug, PartialEq)]
pub enum IrcCommand {
    Unknown = 0,

    Welcome,  // 001
    YourHost, // 002
    Created,  // 003
    MyInfo,   // 004

    NameReply,  // 353
    EndOfNames, // 366
    Motd,       // 372
    MotdStart,  // 375
    MotdEnd,    // 376

    Ping,    // PING
    Cap,     // CAP
    Join,    // JOIN
    PrivMsg, // PRIVMSG

    Max,
}

#[derive(Debug)]
pub struct IrcMessage {
    // Raw IRC data
    pub tags: IrcTags,
    pub source: String,
    pub command: IrcCommand,
    pub channels: Vec<String>,
    pub text: String,
    pub raw: String,

    // Convenience variables
    pub source_name: String,
    pub source_color: u32,
}

#[derive(Debug)]
pub struct IrcTags {
    // User status
    pub is_mod: bool,
    pub is_subscriber: bool,
    pub is_turbo: bool,
    pub is_vip: bool,

    // User familiarity hints
    pub is_first_message: bool,
    pub is_returning_chatter: bool,

    // Highlight redemption
    pub is_highlighted: bool,

    // User identification
    pub color: Option<u32>,
    pub display_name: Option<String>,
    pub user_id: u32,

    // Emotes used in message
    pub emotes: Vec<EmoteEntry>,
}

#[derive(Debug)]
pub struct EmoteEntry {
    pub emote: Emote,
    pub ranges: Vec<(usize, usize)>,
}

impl Display for IrcCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self).unwrap();
        Ok(())
    }
}

impl IrcCommand {
    pub fn from_str(command: &str) -> IrcCommand {
        match command {
            "PING" => IrcCommand::Ping,
            "CAP" => IrcCommand::Cap,
            "JOIN" => IrcCommand::Join,
            "PRIVMSG" => IrcCommand::PrivMsg,

            // Numeric commands
            "001" => IrcCommand::Welcome,
            "002" => IrcCommand::YourHost,
            "003" => IrcCommand::Created,
            "004" => IrcCommand::MyInfo,
            "353" => IrcCommand::NameReply,
            "366" => IrcCommand::EndOfNames,
            "372" => IrcCommand::Motd,
            "375" => IrcCommand::MotdStart,
            "376" => IrcCommand::MotdEnd,

            _ => IrcCommand::Unknown,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Welcome
                | Self::YourHost
                | Self::Created
                | Self::MyInfo
                | Self::Motd
                | Self::MotdStart
                | Self::MotdEnd
        )
    }
}

impl IrcTags {
    fn new() -> Self {
        Self {
            is_mod: false,
            is_subscriber: false,
            is_turbo: false,
            is_vip: false,
            is_first_message: false,
            is_returning_chatter: false,
            is_highlighted: false,
            color: None,
            display_name: None,
            user_id: 0,
            emotes: Vec::new(),
        }
    }
}

impl IrcMessage {
    pub fn from_str(str: &str) -> Result<IrcMessage, Error> {
        let mut buffer = str;
        let mut tags = IrcTags::new();
        let mut source = String::new();

        // Parse tags
        if buffer.starts_with('@') {
            if let Some(i) = buffer.find(' ') {
                let raw_tags = buffer[1..i].to_string();
                buffer = &buffer[i + 1..];

                tags = parse_tags(raw_tags).unwrap_or_else(|| {
                    println!("Failed to parse tags");
                    IrcTags::new()
                });
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Invalid message, failed to parse tags",
                ));
            }
        }

        // Parse source
        if buffer.starts_with(':') {
            if let Some(i) = buffer.find(' ') {
                source.push_str(buffer[..i].trim_start_matches(':'));
                buffer = &buffer[i + 1..];
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Invalid message, failed to parse source",
                ));
            }
        }

        // Parse command and message
        let mut command = String::new();
        let mut text = String::new();
        if let Some(index) = buffer.find(':') {
            command.push_str(buffer[..index].trim());
            text.push_str(&buffer[index + 1..]);
        } else {
            command.push_str(buffer);
        }

        let (command, channels) = parse_command(command);

        // Debug print for tests
        #[cfg(test)]
        {
            println!("----------");
            println!("TAGS: {tags:?}");
            println!("SOURCE: \"{source}\"");
            println!("COMMAND: \"{command}\"");
            println!("CHANNELS: {channels:?}");
            println!("MESSAGE: \"{text}\"");
            println!("RAW: {str}");
        }

        // Post processed variables

        let source_name = if let Some(display_name) = &tags.display_name {
            display_name.clone()
        } else if let Some(i) = source.find('!') {
            source.split_at(i).0.to_string()
        } else {
            source.clone()
        };

        let source_color = get_user_color(&source_name);

        Ok(IrcMessage {
            tags,
            source,
            command,
            channels,
            text,
            raw: str.to_string(),
            source_name,
            source_color,
        })
    }
}

fn parse_command(raw_command: String) -> (IrcCommand, Vec<String>) {
    let mut parts = raw_command.split_whitespace();
    let command = IrcCommand::from_str(parts.next().unwrap());
    let channels = parts.map(|x| x.to_string()).collect();
    (command, channels)
}

fn parse_tags(raw_tags: String) -> Option<IrcTags> {
    let mut tags = IrcTags::new();

    for raw_tag in raw_tags.split(';') {
        if let Some((k, v)) = raw_tag.split_once('=') {
            match k {
                "badge-info" => {}
                "badges" => {}
                "bits" => {}
                "color" => {
                    if v.starts_with('#') {
                        if let Ok(color) = u32::from_str_radix(v.strip_prefix('#').unwrap(), 16) {
                            tags.color = Some(color);
                        }
                    }
                }
                "first-msg" => {
                    tags.is_first_message = v == "1";
                }
                "display-name" => {
                    tags.display_name = Some(v.to_string());
                }
                "emotes" => {
                    if !v.is_empty() {
                        for emote_entry in v.split('/') {
                            let emote_entry_parts = emote_entry.split_once(':')?;
                            if let Ok(emote_id) = emote_entry_parts.0.parse() {
                                let mut ranges = Vec::new();

                                for range in emote_entry_parts.1.split(',') {
                                    let (a, b) = range.split_once('-')?;
                                    let a = a.parse::<usize>().unwrap();
                                    let b = b.parse::<usize>().unwrap();
                                    ranges.push((a, b));
                                }

                                tags.emotes.push(EmoteEntry {
                                    emote: Emote {
                                        server: EmoteServer::Twitch,
                                        id: emote_id,
                                    },
                                    ranges,
                                });
                            }
                        }
                    }
                }
                "mod" => {
                    tags.is_mod = v == "1";
                }
                "msg-id" => {
                    tags.is_highlighted = v == "1";
                }
                "returning-chatter" => {
                    tags.is_returning_chatter = v == "1";
                }
                "subscriber" => {
                    tags.is_subscriber = v == "1";
                }
                "turbo" => {
                    tags.is_turbo = v == "1";
                }
                "user-id" => {
                    tags.user_id = v.parse().unwrap_or(0);
                }
                "vip" => {
                    tags.is_vip = v == "1";
                }
                // Explicitly ignore in case future use is found
                "client-nonce"
                | "flags"
                | "id"
                | "emote-only"
                | "emote-sets"
                | "reply-parent-display-name"
                | "reply-parent-msg-body"
                | "reply-parent-msg-id"
                | "reply-parent-msg-login"
                | "reply-parent-user-id"
                | "reply-parent-user-login"
                | "reply-thread-parent-msg-id"
                | "reply-thread-parent-user-login"
                | "room-id"
                | "tmi-sent-ts"
                | "user-type" => (),
                _ => {
                    println!("Unrecognized tag \"{k}={v}\"");
                }
            }
        }
    }

    Some(tags)
}

#[cfg(test)]
mod unit_tests {
    use super::IrcMessage;

    #[test]
    fn test_message_parser() {
        let cases = [
            "@badge-info=;badges=turbo/1;color=#0D4200;display-name=ronni;emotes=25:0-4,12-16/1902:6-10;id=b34ccfc7-4977-403a-8a94-33c6bac34fb8;mod=0;room-id=1337;subscriber=0;tmi-sent-ts=1507246572675;turbo=1;user-id=1337;user-type=global_mod :ronni!ronni@ronni.tmi.twitch.tv PRIVMSG #ronni :Kappa Keepo Kappa",
            "@badge-info=;badges=staff/1,bits/1000;bits=100;color=;display-name=ronni;emotes=;id=b34ccfc7-4977-403a-8a94-33c6bac34fb8;mod=0;room-id=12345678;subscriber=0;tmi-sent-ts=1507246572675;turbo=1;user-id=12345678;user-type=staff :ronni!ronni@ronni.tmi.twitch.tv PRIVMSG #ronni :cheer100",
            "@badge-info=;badges=vip/1,partner/1;client-nonce=cd15335a5e2059c3b087e22612de485e;color=;display-name=fun2bfun;emotes=;first-msg=0;flags=;id=1fd20412-965f-4c96-beb3-52266448f564;mod=0;returning-chatter=0;room-id=102336968;subscriber=0;tmi-sent-ts=1661372052425;turbo=0;user-id=12345678;user-type=;vip=1 :ronni!ronni@ronni.tmi.twitch.tv PRIVMSG #ronni :Kappa Keepo Kappa",
            "@badge-info=;badges=moderator/1;color=#FF4500;display-name=mybot;emote-sets=0,300374282;mod=1;subscriber=0;user-type=mod :tmi.twitch.tv USERSTATE #bar",
            ":tmi.twitch.tv 001 <user> :Welcome, GLHF!\r\n:tmi.twitch.tv 002 <user> :Your host is tmi.twitch.tv\r\n:tmi.twitch.tv 003 <user> :This server is rather new\r\n:tmi.twitch.tv 004 <user> :-\r\n:tmi.twitch.tv 375 <user> :-\r\n:tmi.twitch.tv 372 <user> :You are in a maze of twisty passages, all alike.\r\n:tmi.twitch.tv 376 <user> :>\r\n@badge-info=;badges=;color=;display-name=<user>;emote-sets=0,300374282;user-id=12345678;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n",
            "PING :tmi.twitch.tv",
        ];

        for case in cases {
            for line in case.lines() {
                let _ = IrcMessage::from_str(line);
            }
        }
    }
}
