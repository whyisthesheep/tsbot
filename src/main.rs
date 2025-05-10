use serenity::{
    utils::parse_emoji,
    async_trait,
    prelude::*,
    all::{
        EmojiId,
        Typing,
        UserId
    },
    model::{
        gateway::GatewayIntents,
        channel::{
            Message,
            ReactionType,
        },
    },
};
use tracing_subscriber::FmtSubscriber;
use tracing::{
    Level,
    instrument,
    error,
    info,
};
use std::env;

struct Handler {
    allowed_id: UserId,
    bot_id: UserId,
    custom_emoji_id: EmojiId,
}
#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, ctx, msg))]
    async fn message(&self, ctx: Context, msg: Message) {
        let allowed = vec![self.allowed_id];

        if !allowed.contains(&msg.author.id) {
            match () {
                _ => {}
            }
            return;
        }

        if allowed.contains(&msg.author.id) {
            match msg.content.as_str() {
                ",sob" => self.handle_sob(&ctx, &msg).await,
                ",broke" => self.handle_broken(&ctx, &msg).await,
                ",delete" => self.handle_delete(&ctx, &msg).await,
                ",pin" => self.handle_pin(&ctx, &msg).await,
                ",get" => self.handle_lance(&ctx, &msg).await,
                ",type" => self.handle_type(&ctx, &msg).await,
                "fire" => self.handle_fire(&ctx, &msg).await,
                "sybau" => self.handle_sybau(&ctx, &msg).await,
                _ => match msg.content.as_str() {
                    _ if msg.content.to_lowercase().contains("lance")
                        && msg.author.id != self.bot_id =>
                    {
                        self.handle_lance(&ctx, &msg).await
                    }
                    _ => {}
                },
            }
        } else {
            match msg.content.as_str() {
                _ if msg.content.to_lowercase().contains("lance")
                    && msg.author.id != self.bot_id =>
                {
                    self.handle_lance(&ctx, &msg).await
                }
                _ => {}
            }
        }
    }
}

impl Handler {
    #[instrument(skip(self, ctx, msg))]
    async fn handle_sob(&self, ctx: &Context, msg: &Message) {
        if let Some(referenced) = &msg.referenced_message {
            let emoji = ReactionType::Unicode('😭'.to_string());
            if let Err(e) = referenced.react(&ctx.http, emoji).await {
                error!("Failed to add reaction: {:?}", e);
            } else {
                info!("Added sob reaction");
            }
        }
        self.safe_delete(ctx, msg).await;
    }

    #[instrument(skip(self, ctx, msg))]
    async fn handle_lance(&self, ctx: &Context, msg: &Message) {
        let emoji_name = "lance";
        let custom_emoji = ReactionType::Custom {
            animated: false,
            id: self.custom_emoji_id,
            name: Some(emoji_name.to_string()),
    };

        if let Err(e) = msg.react(&ctx.http, custom_emoji.clone()).await {
            error!("Failed to react: {:?}", e);
        }

        let response = format!("<:{emoji_name}:{custom_emoji}>");
        if let Err(e) = msg.reply(&ctx.http, response).await {
            error!("Failed to reply: {:?}", e);
        }
    }

    #[instrument(skip(self, ctx, msg))]
    async fn handle_broken(&self, ctx: &Context, msg: &Message) {
        if let Some(referenced) = &msg.referenced_message {
            let emoji = ReactionType::Unicode('💔'.to_string());
            if let Err(e) = referenced.react(&ctx.http, emoji).await {
                error!("Failed to add reaction: {:?}", e);
            } else {
                info!("Added broken heart reaction");
            }
        }
        self.safe_delete(ctx, msg).await;
    }

    #[instrument(skip(self, ctx, msg))]
    async fn handle_delete(&self, ctx: &Context, msg: &Message) {
        if let Some(referenced) = &msg.referenced_message {
            if let Err(e) = referenced.delete(&ctx.http).await {
                error!("Failed to delete message: {:?}", e);
            }
            self.safe_delete(ctx, msg).await;
        }
    }

    #[instrument(skip(self, ctx, msg))]
    async fn handle_sybau(&self, ctx: &Context, msg: &Message) {
        if let Some(referenced) = &msg.referenced_message {
            if let Err(e) = referenced.reply(&ctx.http, "yeah sybau").await {
                error!("Failed to send reply: {:?}", e);
            }
        }
    }

    #[instrument(skip(self, ctx, msg))]
    async fn handle_pin(&self, ctx: &Context, msg: &Message) {
        if let Some(referenced) = &msg.referenced_message {
            if let Err(e) = referenced.pin(&ctx.http).await {
                error!("Failed to pin message: {:?}", e);
            }
        }
    }

    #[instrument(skip(self, ctx, msg))]
    async fn handle_fire(&self, ctx: &Context, msg: &Message) {
        if let Some(referenced) = &msg.referenced_message {
            let emoji = ReactionType::Unicode('🔥'.to_string());
            if let Err(e) = referenced.react(&ctx.http, emoji).await {
                error!("Failed to add fire reaction: {:?}", e);
            }
        }
        self.safe_delete(ctx, msg).await;
    }

    #[instrument(skip(self, ctx, msg))]
    async fn handle_customemoji(&self, ctx: &Context, msg: &Message) {
        let Some(referenced_msg) = &msg.referenced_message else {
            return;
        };

        let emoji_str = referenced_msg
            .content
            .split_whitespace()
            .find_map(|word| parse_emoji(word).map(|_| word.to_string()));

        match emoji_str {
            Some(emoji) => {
                if let Some(emoji_id) = emoji.split(':').last().and_then(|s| s.strip_suffix('>')) {
                    let url = format!("https://cdn.discordapp.com/emojis/{}.png", emoji_id);

                    if let Err(e) = msg.channel_id.say(&ctx.http, url).await {
                        error!("Failed to reply with emoji: {:?}", e);
                    }
                }
            }
            None => {}
        }
        self.safe_delete(ctx, msg).await;
    }

    #[instrument(skip(self, ctx, msg))]
    async fn handle_type(&self, ctx: &Context, msg: &Message) {
        self.safe_delete(ctx, msg).await;
        let typing = Typing::start(ctx.http.clone(), msg.channel_id);
        tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        typing.stop();
    }

    #[instrument(skip(self, ctx, msg))]
    async fn safe_delete(&self, ctx: &Context, msg: &Message) {
        if let Err(e) = msg.delete(&ctx.http).await {
            error!("Failed to delete command message: {:?}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set logging subscriber");

    dotenvy::dotenv().expect("Failed to load .env file");
    let token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN in environment");
    
    let allowed_id = env::var("USERID").expect("Missing USERID in environment")
        .parse::<u64>().expect("USERID must be a number");
    let bot_id = env::var("BOTID").expect("Missing BOTID in environment")
        .parse::<u64>().expect("BOTID must be a number");
    let custom_emoji_id = env::var("CUSTOMEMOJI").expect("Missing CUSTOMEMOJI in environment")
        .parse::<u64>().expect("CUSTOMEMOJI must be a number");

    let handler = Handler {
        allowed_id: UserId::new(allowed_id),
        bot_id: UserId::new(bot_id),
        custom_emoji_id: EmojiId::new(custom_emoji_id),
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");


    info!("Starting bot...");
    if let Err(e) = client.start().await {
        error!("Client error: {:?}", e);
    }
}
