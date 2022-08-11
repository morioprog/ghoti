use clap::Parser;
use cpu::bot::{BeamSearchAI, RandomAI, AI};
use ghoti_simulator::simulate_2p;
use logger::*;

#[derive(Parser)]
#[clap(
    name = "Ghoti CLI (2P)",
    author = "morioprog",
    version = "v0.0.1",
    about = "2人対戦のシミュレータ"
)]
struct Opts {
    /// AI の名前（1P）
    #[clap(long, default_value = "BeamSearchAI")]
    ai_1p: String,

    /// AI の名前（2P）
    #[clap(long, default_value = "BeamSearchAI")]
    ai_2p: String,

    /// 何本先取か
    #[clap(long, default_value = "30")]
    win_goal: usize,

    /// AI に何手読みさせるか
    #[clap(long, default_value = "2")]
    visible_tumos: usize,

    /// Pull Request の ID
    #[clap(long)]
    pr_number: Option<usize>,

    /// 配ぷよ番号
    #[clap(long)]
    haipuyo_margin: Option<usize>,
}

fn main() -> Result<(), std::io::Error> {
    let opts = Opts::parse();

    let ais: Vec<Box<dyn AI>> = vec![Box::new(BeamSearchAI::new()), Box::new(RandomAI::new())];
    let ai_1p = ais
        .iter()
        .find(|&ai| ai.name() == opts.ai_1p)
        .expect(&format!("No AI found: {}", opts.ai_1p));
    let ai_2p = ais
        .iter()
        .find(|&ai| ai.name() == opts.ai_2p)
        .expect(&format!("No AI found: {}", opts.ai_2p));

    let mut logger: Box<dyn Logger> = if opts.pr_number.map(|x| x > 0).unwrap_or(false) {
        Box::new(NullLogger::new("", None)?)
    } else {
        Box::new(FileLogger::new(
            &format!("simulator/logs/cli_2p/{}_vs_{}", ai_1p.name(), ai_2p.name()),
            None,
        )?)
    };

    let simulate_result_2p = simulate_2p(
        &mut logger,
        &Box::new(ai_1p),
        &Box::new(ai_2p),
        opts.win_goal,
        opts.visible_tumos,
        opts.haipuyo_margin,
    );

    // output JSON file
    if let Some(pr_number) = opts.pr_number {
        logger.print("Generating JSON file...".into())?;
        simulate_result_2p?.export_json(pr_number, ai_1p.name(), ai_2p.name())?;
    }

    Ok(())
}
