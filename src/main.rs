use chrono::Local;
use clap::Parser;
use env_logger::{Builder};
use log::{error, info, LevelFilter};
use sim::checker::Checker;
use sim::input_modeling::ContinuousRandomVariable;
use sim::models::Reportable;
use sim::models::{Model, Processor, Storage};
use sim::report::Report;
use sim::simulator::{Connector, Message, Simulation};
use std::io::Write;
/// A command-line application to simulate a ping-pong game with N players.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of players in the simulation
    #[clap(short, long, default_value_t = 2)]
    num_players: usize,

    /// Simulation end time
    #[clap(short, long)]
    end_time: Option<f64>,

    /// Simulation end time
    #[clap(short, long)]
    iterations: Option<usize>,

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

    let mut models: Vec<_> = (0..args.num_players)
        .map(|i| {
            Model::new(
                format!("player-{:02}", i + 1),
                Box::new(Processor::new(
                    ContinuousRandomVariable::Exp { lambda: 0.9 },
                    None,
                    String::from("receive"),
                    String::from("send"),
                    false,
                    None,
                )),
            )
        })
        .collect();

    //build the ping pong ring.
    let mut connectors: Vec<_> = (0..args.num_players)
        .map(|i| {
            let current_player = i + 1;
            let next_player = (i + 1) % args.num_players + 1;
            let source_player = format!("player-{:02}", current_player);
            let target_player = format!("player-{:02}", next_player);
            Connector::new(
                format!("{} to {}", source_player, target_player),
                source_player.clone(),
                target_player.clone(),
                String::from("send"),
                String::from("receive"),
            )
        })
        .collect();

    // not using storage to store the value, but collect the records.
    models.push(Model::new(
        "Store".to_string(),
        Box::new(Storage::new(
            "put".to_string(),
            "get".to_string(),
            "stored".to_string(),
            true,
        )),
    ));

    //from last player to stop port on stats.
    let source_player = format!("player-{:02}", args.num_players);
    let target_player = "Store".to_string();
    connectors.push(Connector::new(
        format!("{} to {}", source_player, target_player),
        source_player.clone(),
        target_player.clone(),
        String::from("send"),
        String::from("put"),
    ));

    let mut simulation = Simulation::post(models, connectors);
    info!("Checking simulation configuration...");
    match simulation.check() {
        Ok(_) => info!("Simulation checks complete"),
        Err(msg) => {
            error!("Check failed: {}", msg);
            std::process::exit(1); // Exit with an error code
        }
    }
    if args.diagram {
        info!("Generating Simulation diagram...");
        let dot_graph = simulation.generate_dot_graph();
        println!("{}", dot_graph);
        // You can save this to a file or pipe it to a graphviz tool like dot
    } else {
        info!("Starting simulation...");

        // Setup a single 'ball' message that will be sent to 'player-01' receive port to start the simulation activity.
        let initial_messages = [Message::new(
            "manual".to_string(),
            "manual".to_string(),
            "player-01".to_string(),
            "receive".to_string(),
            0.0,
            "Ball".to_string(),
        )];

        initial_messages.iter().for_each(|m| {
            info!("injecting initial messages: {:?}", m);
            simulation.inject_input(m.clone())
        });

        // let msgs= match (args.end_time, args.iterations) {
        match (args.end_time, args.iterations) {
            (Some(end_time), _) => simulation.step_until(end_time).unwrap(),
            (_, Some(iterations)) => simulation.step_n(iterations).unwrap(),
            (_,_) => panic!("must provide either 'end_time' or 'iterations'.")
        };
        // println!("Simulation finished with {} messages", msgs.len());
        
        let storage_model = simulation.get_models().get("Store").unwrap();
        println!("round-trip count:{}", &storage_model.records().iter().count())
        // println!("{}", serde_json::to_string(records).unwrap());
        //
        // info!("Simulation complete. Messages: {:?}", msgs);
        // info!("Sim State: {}", serde_json::to_string(&simulation).unwrap());


    }

}
