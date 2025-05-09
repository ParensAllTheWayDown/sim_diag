use chrono::Local;
use clap::Parser;
use env_logger::{Builder, Target};
use log::{LevelFilter, error, info};
use std::io::Write;

use sim::checker::Checker;
use sim::input_modeling::ContinuousRandomVariable;
use sim::models::{Model, Processor};
use sim::report::Report;
use sim::simulator::{Connector, Message, Simulation};
/// A command-line application to simulate a ping-pong game with N players.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of players in the simulation
    #[clap(short, long, default_value_t = 2)]
    num_players: usize,

    /// Simulation end time
    #[clap(short, long, default_value_t = 100.0)]
    end_time: f64,

    /// Generate a diagram of the connected players
    #[clap(long, default_value_t = false)]
    diagram: bool,
}

fn main() {
    let args = Args::parse();

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

    let mut models = Vec::new();
    for i in 0..args.num_players {
        models.push(Model::new(
            format!("player-{:02}", i + 1),
            Box::new(Processor::new(
                ContinuousRandomVariable::Exp { lambda: 0.9 },
                None,
                String::from("receive"),
                String::from("send"),
                false,
                None,
            )),
        ));
    }

    let mut connectors = Vec::new();
    for i in 0..args.num_players {
        let source_player = format!("player-{:02}", i + 1);
        let target_player = format!("player-{:02}", (i + 1) % args.num_players + 1); // Connect to the next player (circular)

        connectors.push(Connector::new(
            format!("{} to {}", source_player, target_player),
            source_player.clone(),
            target_player.clone(),
            String::from("send"),
            String::from("receive"),
        ));
    }

    let initial_messages = [Message::new(
        "manual".to_string(),
        "manual".to_string(),
        "player-01".to_string(),
        "receive".to_string(),
        0.0,
        "Ball".to_string(),
    )];

    let mut simulation = Simulation::post(models, connectors);

    initial_messages.iter().for_each(|m| {
        info!("injecting intial messages: {:?}", m);
        simulation.inject_input(m.clone())
    });

    info!("Checking simulation configuration...");
    match simulation.check() {
        Ok(_) => info!("Simulation checks complete"),
        Err(msg) => {
            error!("Check failed: {}", msg);
            std::process::exit(1); // Exit with an error code
        }
    }

    if args.diagram {
        let dot_graph = simulation.generate_dot_graph();
        println!("{}", dot_graph);
        // You can save this to a file or pipe it to a graphviz tool like dot
    } else {
        info!("Starting simulation...");

        let msgs = simulation.step_until(args.end_time).unwrap();
        info!("Simulation complete. Messages: {:?}", msgs);
        info!("Sim State: {}", serde_json::to_string(&simulation).unwrap());
    }
}
