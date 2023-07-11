mod client;
mod common;

use crate::common::message::IrcCommand;
use client::TwitchIrcClient;
use std::env;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        println!("Expected at least 1 argument for channel_name");
        panic!("Invalid arguments");
    }

    let mut client = TwitchIrcClient::connect_anonymous().await;

    for channel_name in args.iter().skip(1) {
        client.join_channel(channel_name);
    }

    while let Some(m) = client.message_receiver.recv().await {
        // Respond to keep alive messages
        if m.command == IrcCommand::Ping {
            client.send_message(&format!("PONG :{}", m.text));
            continue;
        }

        if m.command == IrcCommand::PrivMsg {
            println!("{}: {}", m.source_name, m.text);
        } else if m.command == IrcCommand::Join {
            println!("\tJoined {}", m.channels.join(", "));
        } else {
            println!("\t{:?} -> {}", m.command, m.raw);
        }
    }
}
