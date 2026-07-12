use bevy::prelude::*;
use strategy_game::simulation::{GameConfig, GameResources, Owner, CityDominance};
use strategy_game::{SimulationPlugin, AiPlugin, setup_initial_map};

fn main() {
    println!("=========================================================================");
    println!("            PACKETCOMMAND AI PARAMETER TUNER & FUN-RATER                 ");
    println!("=========================================================================");
    println!("Sweeping game parameters to find the most engaging and fun balance configuration...");

    // Define search sweeps
    let starting_bws = [100.0, 200.0, 300.0];
    let upgrade_costs = [100.0, 120.0, 150.0];
    let wire_costs = [40.0, 60.0, 80.0];
    let buyout_base_costs = [100.0, 150.0, 200.0];

    let mut best_score = -1;
    let mut best_config = GameConfig::default();
    let mut best_report = String::new();

    let mut run_count = 0;

    for &sbw in &starting_bws {
        for &upg in &upgrade_costs {
            for &wire in &wire_costs {
                for &buyout in &buyout_base_costs {
                    run_count += 1;
                    
                    let config = GameConfig {
                        starting_bandwidth: sbw,
                        router_upgrade_cost: upg,
                        copper_link_cost: wire,
                        router_placement_cost: wire + 10.0,
                        buyout_base_cost: buyout,
                        buyout_scaling_factor: 850.0,
                    };

                    // Run simulated headless game
                    let (score, report) = run_tuning_match(config.clone());
                    
                    if score > best_score {
                        best_score = score;
                        best_config = config;
                        best_report = report;
                    }
                }
            }
        }
    }

    println!("\n=========================================================================");
    println!("                       SWEEP OPTIMIZATION COMPLETE                       ");
    println!("=========================================================================");
    println!("Simulated {} matches successfully.", run_count);
    println!("Best Balance Configuration Found:");
    println!("  - Starting Bandwidth   : {:.1} BW", best_config.starting_bandwidth);
    println!("  - Router Upgrade Cost  : {:.1} BW", best_config.router_upgrade_cost);
    println!("  - Link Wire Cost       : {:.1} BW", best_config.copper_link_cost);
    println!("  - Router Placement Cost: {:.1} BW", best_config.router_placement_cost);
    println!("  - Buyout Base Cost     : {:.1} BW", best_config.buyout_base_cost);
    println!("\nBest Match Dynamics Report:\n{}", best_report);
    println!("=========================================================================\n");
}

/// Simulates a full headless match under a specific configuration and returns the fun score and report.
fn run_tuning_match(config: GameConfig) -> (i32, String) {
    let mut app = App::new();
    app.add_plugins((SimulationPlugin, AiPlugin));
    
    // Override starting config
    app.insert_resource(config.clone());
    app.add_systems(Startup, setup_initial_map);

    // Initial update to run startup systems
    app.update();

    let mut player_elimination_tick = None;
    let mut ai1_elimination_tick = None;
    let mut ai2_elimination_tick = None;
    let mut ai3_elimination_tick = None;

    // Track city ownership changes over time to measure "Tension"
    let mut last_dominant_owners = std::collections::HashMap::new();
    let mut ownership_switches = 0;

    // Run core simulation for 20,000 cycles (approx 5.5 minutes)
    for tick in 1..=20000 {
        app.update();

        // 1. Audit city ownership switches for tension metrics
        let mut city_query = app.world_mut().query::<(Entity, &CityDominance)>();
        for (entity, dom) in city_query.iter(app.world()) {
            let mut current_owner = Owner::Neutral;
            if dom.player_control_pct > 0.5 { current_owner = Owner::Player; }
            else if dom.ai1_control_pct > 0.5 { current_owner = Owner::AI1; }
            else if dom.ai2_control_pct > 0.5 { current_owner = Owner::AI2; }
            else if dom.ai3_control_pct > 0.5 { current_owner = Owner::AI3; }

            let last_owner = last_dominant_owners.entry(entity).or_insert(Owner::Neutral);
            if *last_owner != current_owner {
                if *last_owner != Owner::Neutral && current_owner != Owner::Neutral {
                    // Contested transition from one team to another team!
                    ownership_switches += 1;
                }
                *last_owner = current_owner;
            }
        }

        // 2. Track eliminations
        let res = app.world().resource::<GameResources>();
        if res.player_eliminated && player_elimination_tick.is_none() {
            player_elimination_tick = Some(tick);
        }
        if res.ai1_eliminated && ai1_elimination_tick.is_none() {
            ai1_elimination_tick = Some(tick);
        }
        if res.ai2_eliminated && ai2_elimination_tick.is_none() {
            ai2_elimination_tick = Some(tick);
        }
        if res.ai3_eliminated && ai3_elimination_tick.is_none() {
            ai3_elimination_tick = Some(tick);
        }
    }

    // -------------------------------------------------------------------------
    // "FUN-FACTOR" RATING ALGORITHM
    // -------------------------------------------------------------------------
    // Engagement is measured by tension (city battles) and pacing (gradual eliminations).
    let mut fun_score = 100;
    let mut highlights: Vec<String> = Vec::new();

    // 1. Tension Component (City battles)
    // - No battles: boring, stagnant grid (Penalize heavily)
    // - 1-2 battles: modest engagement.
    // - 3+ battles: highly dynamic, fun network wars.
    if ownership_switches == 0 {
        fun_score -= 30;
        highlights.push("Boring: Cities were never contested. The grid remained stagnant.".to_string());
    } else if ownership_switches <= 2 {
        fun_score -= 10;
        highlights.push(format!("Modest Tension: Only {} city control changes occurred.", ownership_switches));
    } else {
        highlights.push(format!("High Tension: Cities changed hands {} times in active network wars!", ownership_switches));
    }

    // 2. Pacing Component (When do players get bought out?)
    // - If players get bought out immediately when the safety lock lifts (at tick 18000), it's a bit too sudden.
    // - If the game ends with 0 buyouts in 5.5 minutes, it's slightly passive but okay.
    // - Ideal pacing: 1 or 2 buyouts occur gradually between 18,000 and 20,000 ticks.
    let elim_count = [
        player_elimination_tick.is_some(),
        ai1_elimination_tick.is_some(),
        ai2_elimination_tick.is_some(),
        ai3_elimination_tick.is_some(),
    ].iter().filter(|&&e| e).count();

    if elim_count == 0 {
        fun_score -= 15;
        highlights.push("Pacing: Slow match. No subnets were bought out.".to_string());
    } else if elim_count >= 3 {
        fun_score -= 20;
        highlights.push("Imbalanced: Sudden death! Most players were wiped out instantly when lock lifted.".to_string());
    } else {
        fun_score += 15; // Bonus for good pacing!
        highlights.push(format!("Excellent Pacing: {} subnets were bought out gradually, maintaining match engagement.", elim_count));
    }

    // Compile match report summary
    let mut report = format!("  * Fun Score: {}/100\n", fun_score);
    for highlight in highlights {
        report.push_str(&format!("  * {}\n", highlight));
    }

    (fun_score, report)
}
