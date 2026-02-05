use bnet::engine::*;
use bnet::model::*;
use bnet::storage::*;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bnet")]
#[command(about = "BNet governance + token engine", long_about = None)]
struct Cli {
    #[arg(long, default_value = "state.json")]
    state: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long, default_value_t = 0)]
        employees: usize,
    },
    AddHolder {
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: String,
        #[arg(long, default_value_t = 0.0)]
        cash: f64,
    },
    IngestRevenue {
        #[arg(long)]
        gross: f64,
        #[arg(long, default_value_t = 0.0)]
        refund: f64,
        #[arg(long)]
        timestamp: Option<String>,
    },
    EmitWeekly {
        #[arg(long)]
        revenue_total: f64,
        #[arg(long)]
        timestamp: Option<String>,
    },
    Distribute {
        #[arg(long)]
        total_tokens: f64,
        #[arg(long)]
        allocations: String, // format: id:weight,id:weight
    },
    Bid {
        #[arg(long)]
        bidder: String,
        #[arg(long)]
        role: RoleTier,
        #[arg(long)]
        amount: f64,
    },
    ListPosition {
        #[arg(long)]
        listing_id: String,
        #[arg(long)]
        seller: String,
        #[arg(long)]
        role: RoleTier,
        #[arg(long)]
        price: f64,
    },
    BuyPosition {
        #[arg(long)]
        listing_id: String,
        #[arg(long)]
        buyer: String,
    },
    CreateVote {
        #[arg(long)]
        vote_id: String,
        #[arg(long)]
        target_role: RoleTier,
        #[arg(long)]
        target_holder: String,
        #[arg(long, default_value = "performance")]
        reason: String,
    },
    CastVote {
        #[arg(long)]
        vote_id: String,
        #[arg(long)]
        voter: String,
        #[arg(long)]
        weight: f64,
        #[arg(long)]
        approve: bool,
    },
    ResolveVote {
        #[arg(long)]
        vote_id: String,
    },
    AutoValueDropVote {
        #[arg(long)]
        vote_id: String,
        #[arg(long)]
        target_role: RoleTier,
        #[arg(long)]
        target_holder: String,
        #[arg(long)]
        current_price: f64,
    },
    Vote {
        #[arg(long)]
        target_role: RoleTier,
        #[arg(long)]
        target_holder: String,
        #[arg(long)]
        voter: String,
        #[arg(long)]
        weight: f64,
        #[arg(long)]
        approve: bool,
        #[arg(long, default_value = "performance")]
        reason: String,
    },
    SetPrice {
        #[arg(long)]
        price: f64,
    },
    ListHolders,
    StateReport,
    ListMarketplace,
    ListVotes,
    TemplateAllocations {
        #[arg(long)]
        name: String,
    },
    PmCreateTask {
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        dod: String,
        #[arg(long)]
        deliverables: String,
        #[arg(long, default_value_t = 300)]
        max_loc: usize,
    },
    PmReadyTask {
        #[arg(long)]
        id: String,
    },
    PmAssignTask {
        #[arg(long)]
        id: String,
        #[arg(long)]
        assignee: String,
        #[arg(long)]
        role: String,
    },
    PmSubmitReview {
        #[arg(long)]
        id: String,
    },
    PmFinalizeTask {
        #[arg(long)]
        id: String,
        #[arg(long)]
        loc_changed: usize,
        #[arg(long)]
        tests_run: bool,
    },
    ListTasks,
}

fn parse_timestamp(opt: Option<String>) -> DateTime<Utc> {
    opt.and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(Utc::now)
}

fn parse_allocations(raw: &str) -> Vec<WorkAllocation> {
    raw.split(',')
        .filter_map(|pair| {
            let mut parts = pair.split(':');
            let id = parts.next()?.to_string();
            let weight = parts.next()?.parse::<f64>().ok()?;
            Some(WorkAllocation { holder_id: id, weight })
        })
        .collect()
}

fn parse_dod(raw: &str) -> Vec<String> {
    raw.split('|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

fn parse_deliverables(raw: &str) -> Vec<Deliverable> {
    raw.split('|')
        .filter_map(|chunk| {
            let mut parts = chunk.split(':');
            let desc = parts.next()?.trim().to_string();
            let loc = parts.next()?.parse::<usize>().ok()?;
            Some(Deliverable {
                description: desc,
                max_loc: loc,
                tests_required: true,
            })
        })
        .collect()
}

fn default_state() -> CompanyState {
    let now = Utc::now();
    CompanyState {
        holders: HashMap::new(),
        treasury_tokens: 0.0,
        treasury_cash: 0.0,
        positions: vec![],
        employee_count: 0,
        token_price_history: vec![],
        emission_policy: EmissionPolicy {
            weekly_payout_percent: 0.20,
            halving_interval_days: 365,
            genesis: now,
        },
        governance_policy: GovernancePolicy {
            vote_threshold: 2.0 / 3.0,
            value_drop_trigger: 0.20,
            value_window_days: 30,
        },
        votes: HashMap::new(),
        marketplace: vec![],
        tasks: HashMap::new(),
    }
}

fn main() {
    let cli = Cli::parse();
    let path = cli.state;

    match cli.command {
        Commands::Init { employees } => {
            let mut state = default_state();
            state.employee_count = employees;
            ensure_positions(&mut state);
            save_state(&path, &state).expect("save state");
            println!("Initialized state with {} employees", employees);
        }
        Commands::AddHolder { id, name, cash } => {
            let mut state = load_state(&path).expect("load state");
            onboard_holder(&mut state, &id, &name, cash).expect("add holder");
            ensure_positions(&mut state);
            save_state(&path, &state).expect("save state");
            println!("Added holder {}", id);
        }
        Commands::IngestRevenue {
            gross,
            refund,
            timestamp,
        } => {
            let mut state = load_state(&path).expect("load state");
            let event = RevenueEvent {
                timestamp: parse_timestamp(timestamp),
                gross_revenue: gross,
                refund_amount: refund,
            };
            let net = net_revenue(&event);
            state.treasury_cash += net;
            save_state(&path, &state).expect("save state");
            println!("Ingested net revenue: {}", net);
        }
        Commands::EmitWeekly {
            revenue_total,
            timestamp,
        } => {
            let mut state = load_state(&path).expect("load state");
            let minted = run_weekly_emission(&mut state, revenue_total, parse_timestamp(timestamp));
            save_state(&path, &state).expect("save state");
            println!("Minted weekly tokens: {}", minted);
        }
        Commands::Distribute {
            total_tokens,
            allocations,
        } => {
            let mut state = load_state(&path).expect("load state");
            let parsed = parse_allocations(&allocations);
            distribute_tokens(&mut state, &parsed, total_tokens).expect("distribute");
            save_state(&path, &state).expect("save state");
            println!("Distributed {} tokens", total_tokens);
        }
        Commands::Bid {
            bidder,
            role,
            amount,
        } => {
            let mut state = load_state(&path).expect("load state");
            let bid = PromotionBid {
                bidder_id: bidder,
                target_role: role,
                bid_amount: amount,
                timestamp: Utc::now(),
            };
            apply_promotion_bid(&mut state, bid).expect("apply bid");
            save_state(&path, &state).expect("save state");
            println!("Bid accepted");
        }
        Commands::Vote {
            target_role,
            target_holder,
            voter,
            weight,
            approve,
            reason,
        } => {
            let mut state = load_state(&path).expect("load state");
            let vote_id = format!("vote-{}", Utc::now().timestamp());
            create_vote(&mut state, &vote_id, target_role, &target_holder, &reason, Utc::now());
            cast_vote(
                &mut state,
                &vote_id,
                Vote {
                    voter_id: voter,
                    weight,
                    approve,
                },
            )
            .expect("cast vote");

            if resolve_vote_if_passed(&mut state, &vote_id).expect("resolve vote") {
                println!("Vote passed: removed holder from role");
            } else {
                println!("Vote recorded (not yet passed)");
            }

            save_state(&path, &state).expect("save state");
        }
        Commands::CreateVote {
            vote_id,
            target_role,
            target_holder,
            reason,
        } => {
            let mut state = load_state(&path).expect("load state");
            create_vote(&mut state, &vote_id, target_role, &target_holder, &reason, Utc::now());
            save_state(&path, &state).expect("save state");
            println!("Created vote {}", vote_id);
        }
        Commands::CastVote {
            vote_id,
            voter,
            weight,
            approve,
        } => {
            let mut state = load_state(&path).expect("load state");
            cast_vote(
                &mut state,
                &vote_id,
                Vote {
                    voter_id: voter,
                    weight,
                    approve,
                },
            )
            .expect("cast vote");
            save_state(&path, &state).expect("save state");
            println!("Cast vote {}", vote_id);
        }
        Commands::ResolveVote { vote_id } => {
            let mut state = load_state(&path).expect("load state");
            let passed = resolve_vote_if_passed(&mut state, &vote_id).expect("resolve vote");
            save_state(&path, &state).expect("save state");
            println!("Resolved vote {}: passed={}", vote_id, passed);
        }
        Commands::ListPosition {
            listing_id,
            seller,
            role,
            price,
        } => {
            let mut state = load_state(&path).expect("load state");
            create_listing(&mut state, &listing_id, &seller, role, price, Utc::now())
                .expect("create listing");
            save_state(&path, &state).expect("save state");
            println!("Listed position {}", listing_id);
        }
        Commands::BuyPosition { listing_id, buyer } => {
            let mut state = load_state(&path).expect("load state");
            buy_listing(&mut state, &listing_id, &buyer).expect("buy listing");
            save_state(&path, &state).expect("save state");
            println!("Bought position {}", listing_id);
        }
        Commands::AutoValueDropVote {
            vote_id,
            target_role,
            target_holder,
            current_price,
        } => {
            let mut state = load_state(&path).expect("load state");
            record_token_price(&mut state, current_price, Utc::now());
            let triggered = auto_trigger_value_drop_vote(
                &mut state,
                current_price,
                Utc::now(),
                &vote_id,
                target_role,
                &target_holder,
            );
            save_state(&path, &state).expect("save state");
            println!("Value-drop trigger: {}", triggered);
        }
        Commands::SetPrice { price } => {
            let mut state = load_state(&path).expect("load state");
            record_token_price(&mut state, price, Utc::now());
            save_state(&path, &state).expect("save state");
            println!("Price recorded: {}", price);
        }
        Commands::ListHolders => {
            let state = load_state(&path).expect("load state");
            for holder in state.holders.values() {
                println!("{} | {} | tokens: {} | cash: {} | roles: {:?}",
                    holder.id, holder.display_name, holder.tokens, holder.cash, holder.positions);
            }
        }
        Commands::StateReport => {
            let state = load_state(&path).expect("load state");
            println!("Employees: {}", state.employee_count);
            println!("Treasury tokens: {}", state.treasury_tokens);
            println!("Treasury cash: {}", state.treasury_cash);
            println!("Positions: {}", state.positions.len());
            println!("Marketplace listings: {}", state.marketplace.len());
            println!("Votes: {}", state.votes.len());
        }
        Commands::ListMarketplace => {
            let state = load_state(&path).expect("load state");
            for l in state.marketplace.iter().filter(|l| l.active) {
                println!("{} | role: {} | seller: {} | price: {} | active: {}",
                    l.id, l.role, l.seller_id, l.price, l.active);
            }
        }
        Commands::ListVotes => {
            let state = load_state(&path).expect("load state");
            for v in state.votes.values() {
                println!("{} | role: {} | holder: {} | votes: {} | resolved: {}",
                    v.id, v.target_role, v.target_holder, v.votes.len(), v.resolved);
            }
        }
        Commands::TemplateAllocations { name } => {
            if let Some(tpl) = allocation_template(&name) {
                for a in tpl {
                    println!("{}:{}", a.holder_id, a.weight);
                }
            } else {
                println!("Unknown template");
            }
        }
        Commands::PmCreateTask {
            id,
            title,
            summary,
            dod,
            deliverables,
            max_loc,
        } => {
            let mut state = load_state(&path).expect("load state");
            let dod_list = parse_dod(&dod);
            let deliv = parse_deliverables(&deliverables);
            pm_create_task(&mut state, &id, &title, &summary, dod_list, deliv, max_loc);
            save_state(&path, &state).expect("save state");
            println!("Created task {}", id);
        }
        Commands::PmReadyTask { id } => {
            let mut state = load_state(&path).expect("load state");
            pm_ready_task(&mut state, &id).expect("ready task");
            save_state(&path, &state).expect("save state");
            println!("Task ready {}", id);
        }
        Commands::PmAssignTask { id, assignee, role } => {
            let mut state = load_state(&path).expect("load state");
            pm_assign_task(&mut state, &id, &assignee, &role).expect("assign task");
            save_state(&path, &state).expect("save state");
            println!("Assigned task {}", id);
        }
        Commands::PmSubmitReview { id } => {
            let mut state = load_state(&path).expect("load state");
            pm_submit_for_review(&mut state, &id).expect("submit review");
            save_state(&path, &state).expect("save state");
            println!("Task in review {}", id);
        }
        Commands::PmFinalizeTask { id, loc_changed, tests_run } => {
            let mut state = load_state(&path).expect("load state");
            pm_finalize_task(&mut state, &id, loc_changed, tests_run).expect("finalize task");
            save_state(&path, &state).expect("save state");
            println!("Task finalized {}", id);
        }
        Commands::ListTasks => {
            let state = load_state(&path).expect("load state");
            for t in state.tasks.values() {
                println!("{} | {} | status: {:?} | deliverables: {}",
                    t.id, t.title, t.status, t.deliverables.len());
            }
        }
    }
}
