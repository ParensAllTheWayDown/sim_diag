// Code a ping pong simulation.

use chrono::Local;
use env_logger::{Builder, Env, Target};
use log::{debug, error, info, LevelFilter};
use std::io::Write;

use sim::checker::Checker;
use sim::input_modeling::ContinuousRandomVariable;
use sim::models::{Generator, Model, Processor, Storage};
use sim::simulator::{Connector, Message, Simulation};

#[test]
fn test_ping_pong() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let models = [
        Model::new(
            String::from("player-01"),
            Box::new(Processor::new(
                ContinuousRandomVariable::Exp { lambda: 0.9 },
                None,
                String::from("receive"),
                String::from("send"),
                false,
                None,
            )),
        ),
        Model::new(
            String::from("player-02"),
            Box::new(Processor::new(
                ContinuousRandomVariable::Exp { lambda: 0.9 },
                None,
                String::from("receive"),
                String::from("send"),
                false,
                None,
            )),
        ),
    ];

    let connectors = [
        Connector::new(
            String::from("p1 to p2"),
            String::from("player-01"),
            String::from("player-02"),
            String::from("send"),
            String::from("receive"),
        ),
        Connector::new(
            String::from("p2 to p1"),
            String::from("player-02"),
            String::from("player-01"),
            String::from("send"),
            String::from("receive"),
        ),
    ];

    let initial_messages = [Message::new(
        "manual".to_string(),
        "manual".to_string(),
        "player-01".to_string(),
        "receive".to_string(),
        0.0,
        "Ball".to_string(),
    )];

    let mut simulation = Simulation::post(models.to_vec(), connectors.to_vec());

    initial_messages.iter().for_each(|m| {
        info!("injecting intial messages: {:?}", m);
        simulation.inject_input(m.clone())
    });

    info!("Checking simulation configuration...");
    // Check the simulation configuration to verify that it is usable.
    match simulation.check() {
        Ok(_) => info!("Simulation checks complete"),
        Err(msg) => {
            error!("Check failed: {}", msg);
            assert!(false);
        }
    }
    let msgs = simulation.step_until(100.0).unwrap();
    info!("msgs: {:?}", msgs);
    info!("Sim State: {}", serde_json::to_string(&simulation).unwrap());
}
