use crate::common::{message::IrcCommand, IrcMessage, SOCKET_URL};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream};

type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;
type WebSocketSink = SplitSink<WebSocket, Message>;
type WebSocketStream = SplitStream<WebSocket>;

const ANONYMOUS_NICK: &str = "justinfan69";
const ANONYMOUS_PASS: &str = "password";

pub struct TwitchIrcClient {
    message_sender: UnboundedSender<String>,
    pub message_receiver: UnboundedReceiver<IrcMessage>,

    command_ignore_type: [bool; IrcCommand::Max as usize],
}

impl TwitchIrcClient {
    pub async fn connect_anonymous() -> Self {
        // Open web socket connection using tungstenite
        let socket = connect_async(SOCKET_URL)
            .await
            .expect("Failed to connect to Twitch IRC")
            .0;

        // Split socket into read/write
        let (ws_write, ws_read) = socket.split();
        let (rx_write, rx_read) = mpsc::unbounded_channel();
        let (tx_write, tx_read) = mpsc::unbounded_channel();

        let client = TwitchIrcClient {
            message_sender: tx_write,
            message_receiver: rx_read,
            command_ignore_type: [false; IrcCommand::Max as usize],
        };

        tokio::spawn(Self::send_loop(ws_write, tx_read));
        tokio::spawn(Self::receive_loop(ws_read, rx_write));

        client.authenticate(ANONYMOUS_NICK, ANONYMOUS_PASS);
        client
    }

    pub fn authenticate(&self, nick: &str, password: &str) {
        self.send_message("CAP REQ :twitch.tv/tags");
        self.send_message(&format!("PASS {password}"));
        self.send_message(&format!("NICK {nick}"));
    }

    pub fn join_channel(&self, channel_name: &str) {
        self.send_message(&format!("JOIN #{channel_name}"));
    }

    pub fn send_message(&self, text: &str) {
        self.message_sender.send(text.to_string()).unwrap();
    }

    async fn send_loop(mut ws_write: WebSocketSink, mut tx_read: UnboundedReceiver<String>) {
        while let Some(string) = tx_read.recv().await {
            let message = Message::text(string);
            ws_write.feed(message).await.unwrap();
            ws_write.flush().await.unwrap();
        }
    }

    async fn receive_loop(mut ws_read: WebSocketStream, rx_write: UnboundedSender<IrcMessage>) {
        while let Some(item) = ws_read.next().await {
            if let Ok(message) = item {
                if let Ok(text) = message.to_text() {
                    text.lines()
                        .flat_map(IrcMessage::from_str)
                        .filter(|m| !m.command.is_numeric())
                        .for_each(|m| rx_write.send(m).unwrap());
                }
            }
        }
    }

    fn is_ignored(&self, command_type: IrcCommand) -> bool {
        let i = command_type as usize;
        self.command_ignore_type[i]
    }
}
