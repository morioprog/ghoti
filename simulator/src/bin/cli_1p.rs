use clap::Parser;
use cpu::bot::{RandomAI, AI};
use logger::*;
use simulator::simulate_1p;

#[derive(Parser)]
#[clap(
    name = "Ghoti CLI (1P)",
    author = "morioprog",
    version = "v0.0.1",
    about = "とこぷよのシミュレータ"
)]
struct Opts {
    /// AI の名前（`ai.name()`）
    #[clap(long, default_value = "RandomAI")]
    ai: String,

    /// 最大手数
    #[clap(long, default_value = "100")]
    max_tumos: usize,

    /// AI に何手読みさせるか
    #[clap(long, default_value = "2")]
    visible_tumos: usize,

    /// とこぷよをする回数
    #[clap(long, default_value = "1")]
    trial: usize,

    /// Pull Request の ID
    #[clap(long)]
    pr_number: Option<usize>,

    /// 配ぷよ番号
    #[clap(long)]
    haipuyo_margin: Option<usize>,

    /// この得点以上の連鎖が打たれたら終了
    #[clap(long)]
    required_chain_score: Option<usize>,
}

fn main() -> Result<(), std::io::Error> {
    let opts = Opts::parse();

    let ais: Vec<Box<dyn AI>> = vec![Box::new(RandomAI::new())];
    let ai = ais
        .iter()
        .find(|&ai| ai.name() == opts.ai)
        .expect(&format!("No AI found: {}", opts.ai));

    let mut logger: Box<dyn Logger> = match opts.pr_number {
        None => Box::new(FileLogger::new(
            &format!("simulator/logs/cli_1p/{}", ai.name()),
            None,
        )?),
        Some(_) => Box::new(NullLogger::new("", None)?),
    };

    // Repeat these whole step `opts.trial` times
    for trial_index in 1..=opts.trial {
        if opts.pr_number.is_some() {
            println!("Generating JSON... ({})", trial_index);
        }

        let simulate_result_1p = simulate_1p(
            &mut logger,
            ai,
            opts.visible_tumos,
            opts.max_tumos,
            opts.haipuyo_margin,
            opts.required_chain_score,
        );

        // output JSON file
        if let Some(pr_number) = opts.pr_number {
            logger.print("\nGenerating JSON file...".into())?;
            simulate_result_1p?.export_json(pr_number, ai.name())?;
        }
    }

    Ok(())
}
