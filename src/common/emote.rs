use std::fmt::Display;

#[derive(Debug)]
pub enum EmoteServer {
    Twitch,
    Bttv,
    FrankerFaceZ,
    SevenTv,
}

#[derive(Debug)]
pub struct Emote {
    pub server: EmoteServer,
    pub id: u32,
}

impl Display for Emote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}:{}", self.server, self.id))
            .unwrap();
        Ok(())
    }
}

// TODO:
// Add functions to get emote sets and cache emotes from broadcaster id
// using all supported emote services
// https://api.twitch.tv/helix/chat/emotes?broadcaster_id={}

#[macro_export]
macro_rules! emoteurl {
    /*
       Twitch template url:
       "https://static-cdn.jtvnw.net/emoticons/v2/{{id}}/{{format}}/{{theme_mode}}/{{scale}}"
    */
    (emote_server: EmoteServer, emote_id: &str) => {
        println!("This macro probably needs some work, or should be converted to a function");
        match emote_server {
            Twitch => {
                format!("https://static-cdn.jtvnw.net/emoticons/v2/{emote_id}/default/dark/1.0",)
            }
            Bttv => format!("https://cdn.betterttv.net/emote/{emote_id}/1x"),
            FrankerFaceZ => todo!("FrankerFaceZ emotes not implemented yet"),
            SevenTv => todo!("7tv emotes not implemented yet"),
        }
    };
}
