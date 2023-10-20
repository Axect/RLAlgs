use indicatif::{ProgressBar, ProgressStyle};
use peroxide::fuga::*;
use rlai::base::policy::{EpsilonGreedyValuePolicy, Policy};
use rlai::base::process::MarkovDecisionProcess;
use rlai::env::grid_world::GridWorld;
use rlai::learning::util::ConstantStepsize;
use rlai::learning::value_prediction::EveryvisitMC;
use std::collections::HashMap;

fn main() {
    let env = GridWorld::new(5, 5, (0, 0), (4, 3), vec![(2, 4), (4, 0)]);
    let stepsize_scheduler = ConstantStepsize::new(0.01);
    let mut policy = EpsilonGreedyValuePolicy::new(&env, HashMap::new(), 0.1);
    let mut value_predictor: EveryvisitMC<(usize, usize)> =
        EveryvisitMC::new(HashMap::new(), Box::new(stepsize_scheduler), 0.9);

    let mut episodes = vec![];
    let pb = ProgressBar::new(1000);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    for _ in 0..1000 {
        // 1. Generate an episode
        let mut episode = Vec::new();
        let mut current_state = env.get_current_state();
        loop {
            let action = policy.gen_action(&current_state).unwrap();
            match env.step(&current_state, &action) {
                (None, r) => {
                    episode.push((current_state, r));
                    break;
                }
                (Some(s), r) => {
                    episode.push((current_state, r));
                    current_state = s;
                }
            }
        }

        // 2. Compute return via Every-visit MC
        value_predictor.update_episode(&episode);
        value_predictor.step();
        let new_value_function = value_predictor.get_value_function();

        // 3. Update Policy
        policy.update_value_function(new_value_function);

        pb.inc(1);
        pb.set_message(format!("Episode length: {}", episode.len()));

        episodes.push(episode);
    }

    // Test
    // - Turn off random policy
    policy.turn_off_random();
    let mut test_episode = Vec::new();
    let mut current_state = env.get_current_state();
    loop {
        let action = policy.gen_action(&current_state).unwrap();
        match env.step(&current_state, &action) {
            (None, r) => {
                test_episode.push((current_state, r));
                break;
            }
            (Some(s), r) => {
                test_episode.push((current_state, r));
                current_state = s;
            }
        }
    }

    println!("Test Episode: {:?}", test_episode);

    // Store first episodes
    let mut df = DataFrame::new(vec![]);
    let first_episode = episodes[0].clone();
    let ((episode_x, episode_y), rewards): ((Vec<usize>, Vec<usize>), Vec<f64>) =
        first_episode.into_iter().unzip();
    df.push(
        "episode_x",
        Series::new(episode_x.into_iter().map(|x| x as u64).collect()),
    );
    df.push(
        "episode_y",
        Series::new(episode_y.into_iter().map(|x| x as u64).collect()),
    );
    df.push("reward", Series::new(rewards));
    df.write_parquet(
        "./data/grid_world/mc-epsilon_greedy-first.parquet",
        CompressionOptions::Uncompressed,
    )
    .expect("Can't write parquet file");

    // Store test episode
    let mut df = DataFrame::new(vec![]);
    let ((episode_x, episode_y), rewards): ((Vec<usize>, Vec<usize>), Vec<f64>) =
        test_episode.into_iter().unzip();
    df.push(
        "episode_x",
        Series::new(episode_x.into_iter().map(|x| x as u64).collect()),
    );
    df.push(
        "episode_y",
        Series::new(episode_y.into_iter().map(|x| x as u64).collect()),
    );
    df.push("reward", Series::new(rewards));
    df.write_parquet(
        "./data/grid_world/mc-epsilon_greedy-test.parquet",
        CompressionOptions::Uncompressed,
    )
    .expect("Can't write parquet file");

    // Store all episodes' length
    let mut df = DataFrame::new(vec![]);
    df.push(
        "length",
        Series::new(
            episodes
                .iter()
                .map(|e| e.len() as u64)
                .collect::<Vec<u64>>(),
        ),
    );
    df.write_parquet(
        "./data/grid_world/mc-epsilon_greedy-length.parquet",
        CompressionOptions::Uncompressed,
    )
    .expect("Can't write parquet file");
    df.print();
}
