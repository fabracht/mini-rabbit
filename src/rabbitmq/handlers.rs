use actix::{ActorFutureExt, AsyncContext, ContextFutureSpawner, Handler, WrapFuture};
// use futures_util::TryStreamExt;
use futures::TryStreamExt;
use log::{info, warn};

use crate::Rabbit;
use crate::rabbitmq::messages::{ConsumeFromExchange, CreateExchange, PublishToTopic};

impl Handler<CreateExchange> for Rabbit {
    type Result = ();

    fn handle(&mut self, msg: CreateExchange, ctx: &mut Self::Context) -> Self::Result {
        warn!("Handling create exchange message");
        let channel = self.inbound_channel.clone();
        async move {
            Rabbit::declare_exchange(channel, msg).await;
        }.into_actor(self).wait(ctx);
    }
}

impl Handler<PublishToTopic> for Rabbit {
    type Result = ();

    fn handle(&mut self, msg: PublishToTopic, ctx: &mut Self::Context) -> Self::Result {
        info!("Publishing message to {}", msg.routing_key);
        let channel = self.inbound_channel.clone();
        Rabbit::publish_to_topic(channel, msg)
            .into_actor(self)
            .map(|_, _, _| {})
            .wait(ctx);
    }
}


impl Handler<ConsumeFromExchange> for Rabbit {
    type Result = ();
    fn handle(&mut self, msg: ConsumeFromExchange, ctx: &mut Self::Context) -> Self::Result {
        info!("Consuming from exchange");
        let channel = self.inbound_channel.clone();
        Rabbit::consume_from_exchange(channel, msg)
            .into_actor(self)
            .map(|r, _a, c| {
                if let Ok(consumer) = r {
                    let try_stream = consumer.into_stream();
                    info!("Adding Rabbit stream" );
                    c.add_stream(try_stream);
                }
            })
            .wait(ctx);
    }
}