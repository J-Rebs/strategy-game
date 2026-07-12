use bevy::prelude::*;
use strategy_game::simulation::{GameResources, CityDominance, BUYOUT_LOCK_TICKS};
use strategy_game::{SimulationPlugin, AiPlugin, setup_initial_map};

fn main() {
    println!("=========================================================================");
    println!("          PACKETCOMMAND HEADLESS MATCH SIMULATOR & RATER                 ");
    println!("=========================================================================");
    println!("Running 24,000 cycles (~6.6 minutes of simulated play) at maximum speed...");

    let mut app = App::new();
    
    // Add only simulation and AI plugins (omitting rendering and hud)
    app.add_plugins((SimulationPlugin, AiPlugin));
    
    // Register initial map spawner on Startup stage
    app.add_systems(Startup, setup_initial_map);

    // Initial update to run the Startup stage and spawn entities
    app.update();

    let mut player_elimination_tick = None;
    let mut ai1_elimination_tick = None;
    let mut ai2_elimination_tick = None;
    let mut ai3_elimination_tick = None;

    // Run core simulation ticks
    for tick in 1..=24000 {
        app.update();

        // Fetch resources state
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

    // Retrieve final resources and city control shares
    let final_res = app.world().resource::<GameResources>().clone();
    
    let mut city_query = app.world_mut().query::<&CityDominance>();
    let mut total_player_control = 0.0;
    let mut total_ai1_control = 0.0;
    let mut total_ai2_control = 0.0;
    let mut total_ai3_control = 0.0;
    let mut city_count = 0;

    for dom in city_query.iter(app.world()) {
        total_player_control += dom.player_control_pct;
        total_ai1_control += dom.ai1_control_pct;
        total_ai2_control += dom.ai2_control_pct;
        total_ai3_control += dom.ai3_control_pct;
        city_count += 1;
    }

    let avg_player_ctrl = (total_player_control / city_count as f32) * 100.0;
    let avg_ai1_ctrl = (total_ai1_control / city_count as f32) * 100.0;
    let avg_ai2_ctrl = (total_ai2_control / city_count as f32) * 100.0;
    let avg_ai3_ctrl = (total_ai3_control / city_count as f32) * 100.0;

    // -------------------------------------------------------------------------
    // BALANCE RATING CALCULATION
    // -------------------------------------------------------------------------
    let mut rating_score = 100;
    let mut issues = Vec::new();

    // Issue 1: Buyout safety lock violation
    let mut early_buyout_detected = false;
    for &t in &[player_elimination_tick, ai1_elimination_tick, ai2_elimination_tick, ai3_elimination_tick] {
        if let Some(tick) = t {
            if tick < BUYOUT_LOCK_TICKS {
                early_buyout_detected = true;
            }
        }
    }
    if early_buyout_detected {
        rating_score -= 40;
        issues.push("CRITICAL: A buyout occurred before the 5-minute safety lock (18,000 ticks) expired!");
    }

    // Issue 2: One-sided domination
    let controls = [avg_player_ctrl, avg_ai1_ctrl, avg_ai2_ctrl, avg_ai3_ctrl];
    let max_control = controls.iter().cloned().fold(0.0_f32, f32::max);
    if max_control > 75.0 {
        rating_score -= 20;
        issues.push("Warning: One player dominates over 75% of the city network, leading to imbalance.");
    }

    // Issue 3: Stagnant play (no eliminations at all)
    let eliminations_count = [
        final_res.player_eliminated,
        final_res.ai1_eliminated,
        final_res.ai2_eliminated,
        final_res.ai3_eliminated,
    ].iter().filter(|&&e| e).count();

    if eliminations_count == 0 {
        rating_score -= 15;
        issues.push("Notice: No players were bought out during the 6.6-minute run. Costs might be high or AI too passive.");
    }

    // Output Report
    println!("\n=========================================================================");
    println!("                     SIMULATION METRICS & REPORT                         ");
    println!("=========================================================================");
    println!("Final Bandwidth Accounts:");
    println!("  - Player: {:.1} BW (Eliminated: {})", final_res.player_bandwidth, final_res.player_eliminated);
    println!("  - AI 1  : {:.1} BW (Eliminated: {})", final_res.ai1_bandwidth, final_res.ai1_eliminated);
    println!("  - AI 2  : {:.1} BW (Eliminated: {})", final_res.ai2_bandwidth, final_res.ai2_eliminated);
    println!("  - AI 3  : {:.1} BW (Eliminated: {})", final_res.ai3_bandwidth, final_res.ai3_eliminated);
    println!("\nAverage City Network Share:");
    println!("  - Player: {:.1}%", avg_player_ctrl);
    println!("  - AI 1  : {:.1}%", avg_ai1_ctrl);
    println!("  - AI 2  : {:.1}%", avg_ai2_ctrl);
    println!("  - AI 3  : {:.1}%", avg_ai3_ctrl);
    println!("\nElimination Log:");
    println!("  - Player: {:?}", player_elimination_tick.map(|t| format!("Cycle {}", t)).unwrap_or_else(|| "Active".to_string()));
    println!("  - AI 1  : {:?}", ai1_elimination_tick.map(|t| format!("Cycle {}", t)).unwrap_or_else(|| "Active".to_string()));
    println!("  - AI 2  : {:?}", ai2_elimination_tick.map(|t| format!("Cycle {}", t)).unwrap_or_else(|| "Active".to_string()));
    println!("  - AI 3  : {:?}", ai3_elimination_tick.map(|t| format!("Cycle {}", t)).unwrap_or_else(|| "Active".to_string()));
    
    println!("\n=========================================================================");
    println!("                       GAME BALANCE RATING: {}/100                        ", rating_score);
    println!("=========================================================================");
    if issues.is_empty() {
        println!("Status: Excellent game balance! Starting resources and safety locks are correctly configured.");
    } else {
        println!("Status: Tuning Required.");
        for issue in &issues {
            println!("  - {}", issue);
        }
    }
    println!("=========================================================================\n");
}
