use actix::Actor;
use actix_rt::System;
use dotenv::dotenv;
use lapin::ExchangeKind;
use lapin::options::ExchangeDeclareOptions;

use crate::rabbitmq::connection::Rabbit;
use crate::rabbitmq::messages::CreateExchange;

mod rabbitmq;

fn main() {
////////////////////////////////////////////////////////////////////////////
    std::env::set_var("RUST_LOG", "actix_web=info, info");
    std::env::set_var("RUST_LOG_STYLE", "actix_web=info, info");

    ////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////
    /////////// LOG4RS
    log4rs::init_file("logging_config.yaml", Default::default()).unwrap();
    let _path = dotenv().ok();
    let address =
        std::env::var("AMQP_ADDR").expect("You're supposed to provide your own RABBIT cluster");


    ////////////////////////////////////////////////////////////////////////////
    /////////// MAIN FUTURE
    let future1 = async {
        let rabbit = Rabbit::new(&address).await;
        let rabbit_address = rabbit.start();
        let declare_options = ExchangeDeclareOptions {
            passive: false,
            durable: true,
            auto_delete: false,
            internal: false,
            nowait: false,
        };

        let create_exchange = CreateExchange {
            exchange_name: "demo_exchange".to_string(),
            exchange_options: Some(declare_options),
            exchange_kind: ExchangeKind::Topic,
        };
        let _ = rabbit_address.send(create_exchange).await;
    };

    ////////////////////////////////////////////////////////////////////////////
    /////////// RUN THE SYSTEM
    let system = System::new();
    system.block_on(future1);
    let _ = system.run();
}
