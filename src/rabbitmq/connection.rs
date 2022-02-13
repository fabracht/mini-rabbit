use std::str;
use std::time::Duration;

use actix::{Actor, ActorContext, ActorFutureExt, AsyncContext, Context, ContextFutureSpawner, fut, Running, StreamHandler, WrapFuture};
use actix::io::WriteHandler;
use lapin::{BasicProperties, Channel, Connection, Consumer, types::FieldTable};
use lapin::message::Delivery;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueBindOptions, QueueDeclareOptions};
use log::{error, info, warn};

use crate::rabbitmq::messages::{ConsumeFromExchange, CreateExchange, PublishToTopic};

////////////////////////////////////////////////////////////////////////////
/////////// Rabbit Struct holds the inbound channel and the outbound channel
pub struct Rabbit {
    pub inbound_channel: Channel,
    pub outbound_channel: Channel,
    pub connection: Connection,
}

impl Rabbit {
    pub async fn new(address: &str) -> Self {
        let connection =
            lapin::Connection::connect(address, lapin::ConnectionProperties::default())
                .await
                .map_err(|e| {
                    panic!("{:?}", e);
                }).unwrap();

        let inbound_channel = connection.create_channel().await.unwrap();
        let outbound_channel = connection.create_channel().await.unwrap();
        info!("Created Rabbit Instance");
        Self {
            inbound_channel,
            outbound_channel,
            connection,
        }
    }

    pub async fn publish_to_topic(channel: Channel, msg: PublishToTopic) {
        let paymsg = msg.payload.as_bytes().to_vec();
        let channel = channel.basic_publish(
            msg.exchange.as_str(),
            msg.payload.as_str(),
            BasicPublishOptions::default(),
            &paymsg,
            BasicProperties::default(),
        ).await.unwrap();
        channel.await.unwrap();
    }

    pub async fn declare_exchange(channel: Channel, msg: CreateExchange) {
        warn!("Declaring exchange");
        channel
            .exchange_declare(
                &msg.exchange_name,
                msg.exchange_kind.to_owned(),
                msg.exchange_options.unwrap(),
                FieldTable::default(),
            ).await.unwrap();
    }

    pub async fn consume_from_exchange(channel: Channel, msg: ConsumeFromExchange) -> lapin::Result<Consumer> {
        let queue_name = &msg.queue.to_string();
        let exchange = &msg.exchange.to_string();
        info!("Declaring queue {}", queue_name);
        let queue_options = QueueDeclareOptions {
            passive: false,
            durable: false,
            exclusive: true,
            auto_delete: true,
            nowait: true,
        };

        let _queue = channel
            .queue_declare(
                queue_name,
                queue_options,
                FieldTable::default(),
            )
            .await
            .unwrap();
        info!("Binding queue to route {:?}", msg.connection_name);

        let _bind = channel
            .queue_bind(
                queue_name,
                exchange,
                msg.connection_name.as_str(),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
        info!("Basic consumer");
        channel
            .basic_consume(
                queue_name,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
    }
}

impl Actor for Rabbit {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(1000), |act, inner_ctx| {
            let channel = act.inbound_channel.clone();

            Rabbit::publish_to_topic(channel, PublishToTopic {
                exchange: "demo_exchange".to_string(),
                routing_key: "/rabbit".to_string(),
                payload: "rabbit says hi".to_string(),
            })
                .into_actor(act)
                .map(|_, _, _| {}).wait(inner_ctx);
        });

        ctx.notify(ConsumeFromExchange {
            queue: "rabbit_queue".to_string(),
            exchange: "demo_exchange".to_string(),
            connection_name: "/rabbit".to_string(),
            self_id: Default::default(),
            room_id: Default::default(),
        });
        info!("Rabbit Actor Started");
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        warn!("Stopping Rabbit client");
        Running::Stop
    }
}

impl WriteHandler<lapin::Error> for Rabbit {}

impl StreamHandler<Result<Delivery, lapin::Error>> for Rabbit {
    fn handle(&mut self, item: Result<Delivery, lapin::Error>, ctx: &mut Self::Context) {
        info!("Handling stream" );
        match item {
            Ok(delivery) => {
                let delv = delivery.data.clone();
                let payload = str::from_utf8(delv.as_slice()).expect("Failed parsing delivery as utf8");
                let ack = async move {
                    delivery.ack(BasicAckOptions::default()).await.unwrap()
                };
                let _basic_ack = ack.into_actor(self).then(|_, _, _| { fut::ready(()) }).wait(ctx);

                ////////////////////////////////////////////////
                ////////// WRITING TO CACHE

                info!("Rabbit payload received, {}", payload);
            }
            Err(e) => {
                error!("{}", e);
                ctx.stop();
            }
        }
    }
}