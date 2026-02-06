use crate::model::*;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

pub fn net_revenue(event: &RevenueEvent) -> f64 {
    (event.gross_revenue - event.refund_amount).max(0.0)
}

pub fn aggregate_weekly_revenue(events: &[RevenueEvent]) -> f64 {
    events.iter().map(net_revenue).sum()
}

pub fn halving_factor(policy: &EmissionPolicy, now: DateTime<Utc>) -> f64 {
    let days = (now - policy.genesis).num_days();
    let halvings = days / policy.halving_interval_days;
    0.5_f64.powi(halvings as i32)
}

pub fn weekly_emission(policy: &EmissionPolicy, week_revenue: f64, now: DateTime<Utc>) -> f64 {
    let base = week_revenue * policy.weekly_payout_percent;
    base * halving_factor(policy, now)
}

pub fn required_positions(employee_count: usize) -> HashMap<RoleTier, usize> {
    let managers = employee_count / 10;
    let senior_managers = managers / 10;
    let directors = managers / 10; // consistent chain
    let csuite = directors / 5;
    let board_seats = employee_count / 10_000;

    HashMap::from([
        (RoleTier::Manager, managers),
        (RoleTier::SeniorManager, senior_managers),
        (RoleTier::Director, directors),
        (RoleTier::CSuite, csuite),
        (RoleTier::BoardSeat, board_seats),
        (RoleTier::CEO, 1),
        (RoleTier::President, 1),
        (RoleTier::CoPresident, 1),
    ])
}

pub fn ensure_positions(state: &mut CompanyState) {
    let required = required_positions(state.employee_count);
    for (tier, count) in required {
        let existing = state.positions.iter().filter(|p| p.tier == tier).count();
        if existing < count {
            for _ in existing..count {
                state.positions.push(RolePosition {
                    tier,
                    holder_id: None,
                    acquired_at: None,
                    price_paid: None,
                });
            }
        }
    }
}

pub fn run_weekly_emission(state: &mut CompanyState, week_revenue: f64, now: DateTime<Utc>) -> f64 {
    let minted = weekly_emission(&state.emission_policy, week_revenue, now);
    state.treasury_tokens += minted;
    minted
}

pub fn record_token_price(state: &mut CompanyState, price: f64, now: DateTime<Utc>) {
    state.token_price_history.push((now, price));
    // keep only 30 days of history
    let cutoff = now - Duration::days(state.governance_policy.value_window_days);
    state.token_price_history
        .retain(|(ts, _)| *ts >= cutoff);
}

pub fn rolling_average_price(state: &CompanyState) -> Option<f64> {
    if state.token_price_history.is_empty() {
        return None;
    }
    let sum: f64 = state.token_price_history.iter().map(|(_, p)| p).sum();
    Some(sum / state.token_price_history.len() as f64)
}

pub fn value_drop_triggered(state: &CompanyState, current_price: f64) -> bool {
    if let Some(avg) = rolling_average_price(state) {
        let drop = (avg - current_price) / avg;
        drop >= state.governance_policy.value_drop_trigger
    } else {
        false
    }
}

pub fn apply_promotion_bid(state: &mut CompanyState, bid: PromotionBid) -> Result<(), String> {
    let bidder = state
        .holders
        .get_mut(&bid.bidder_id)
        .ok_or("Bidder not found")?;
    if bidder.cash < bid.bid_amount {
        return Err("Insufficient cash for bid".into());
    }

    let position = state
        .positions
        .iter_mut()
        .find(|p| p.tier == bid.target_role && p.holder_id.is_none())
        .ok_or("No available position for this role")?;

    bidder.cash -= bid.bid_amount;
    state.treasury_cash += bid.bid_amount;

    position.holder_id = Some(bidder.id.clone());
    position.acquired_at = Some(bid.timestamp);
    position.price_paid = Some(bid.bid_amount);

    bidder.positions.push(bid.target_role);
    Ok(())
}

pub fn distribute_tokens(
    state: &mut CompanyState,
    allocations: &[WorkAllocation],
    total_tokens: f64,
) -> Result<(), String> {
    let total_weight: f64 = allocations.iter().map(|a| a.weight).sum();
    if total_weight <= 0.0 {
        return Err("Total allocation weight must be > 0".into());
    }
    if state.treasury_tokens < total_tokens {
        return Err("Insufficient treasury tokens".into());
    }

    for allocation in allocations {
        let share = (allocation.weight / total_weight) * total_tokens;
        let holder = state
            .holders
            .get_mut(&allocation.holder_id)
            .ok_or("Holder not found")?;
        holder.tokens += share;
    }
    state.treasury_tokens -= total_tokens;
    Ok(())
}

pub fn submit_vote(record: &mut VoteRecord, vote: Vote) {
    record.votes.push(vote);
}

pub fn vote_passed(record: &VoteRecord, threshold: f64) -> bool {
    let total_weight: f64 = record.votes.iter().map(|v| v.weight).sum();
    if total_weight == 0.0 {
        return false;
    }
    let approve_weight: f64 = record
        .votes
        .iter()
        .filter(|v| v.approve)
        .map(|v| v.weight)
        .sum();
    (approve_weight / total_weight) >= threshold
}

pub fn remove_holder_from_role(state: &mut CompanyState, role: RoleTier, holder_id: &str) {
    if let Some(pos) = state
        .positions
        .iter_mut()
        .find(|p| p.tier == role && p.holder_id.as_deref() == Some(holder_id))
    {
        pos.holder_id = None;
        pos.acquired_at = None;
        pos.price_paid = None;
    }
    if let Some(holder) = state.holders.get_mut(holder_id) {
        holder.positions.retain(|r| *r != role);
    }
}

pub fn onboard_holder(state: &mut CompanyState, id: &str, name: &str, cash: f64) -> Result<(), String> {
    if state.holders.contains_key(id) {
        return Err("Holder already exists".into());
    }
    state.holders.insert(
        id.to_string(),
        Holder {
            id: id.to_string(),
            display_name: name.to_string(),
            tokens: 0.0,
            cash,
            positions: vec![RoleTier::Employee],
        },
    );
    state.employee_count += 1;
    Ok(())
}

pub fn create_listing(
    state: &mut CompanyState,
    listing_id: &str,
    seller_id: &str,
    role: RoleTier,
    price: f64,
    now: DateTime<Utc>,
) -> Result<(), String> {
    let holder = state
        .holders
        .get(seller_id)
        .ok_or("Seller not found")?;
    if !holder.positions.contains(&role) {
        return Err("Seller does not hold that role".into());
    }
    state.marketplace.push(PositionListing {
        id: listing_id.to_string(),
        role,
        seller_id: seller_id.to_string(),
        price,
        created_at: now,
        active: true,
    });
    Ok(())
}

pub fn buy_listing(state: &mut CompanyState, listing_id: &str, buyer_id: &str) -> Result<(), String> {
    let listing = state
        .marketplace
        .iter_mut()
        .find(|l| l.id == listing_id && l.active)
        .ok_or("Listing not found")?;

    let price = listing.price;
    let seller_id = listing.seller_id.clone();
    let role = listing.role;

    {
        let buyer = state
            .holders
            .get_mut(buyer_id)
            .ok_or("Buyer not found")?;
        if buyer.cash < price {
            return Err("Buyer has insufficient cash".into());
        }
        buyer.cash -= price;
        buyer.positions.push(role);
    }

    {
        let seller = state
            .holders
            .get_mut(&seller_id)
            .ok_or("Seller not found")?;
        seller.cash += price;
        seller.positions.retain(|r| *r != role);
    }

    if let Some(pos) = state.positions.iter_mut().find(|p| {
        p.tier == role && p.holder_id.as_deref() == Some(&seller_id)
    }) {
        pos.holder_id = Some(buyer_id.to_string());
        pos.acquired_at = Some(Utc::now());
        pos.price_paid = Some(price);
    }

    listing.active = false;
    Ok(())
}

pub fn create_vote(state: &mut CompanyState, id: &str, role: RoleTier, holder: &str, reason: &str, now: DateTime<Utc>) {
    state.votes.insert(
        id.to_string(),
        VoteRecord {
            id: id.to_string(),
            target_role: role,
            target_holder: holder.to_string(),
            reason: reason.to_string(),
            created_at: now,
            votes: vec![],
            resolved: false,
        },
    );
}

pub fn cast_vote(state: &mut CompanyState, vote_id: &str, vote: Vote) -> Result<bool, String> {
    let record = state.votes.get_mut(vote_id).ok_or("Vote not found")?;
    if record.resolved {
        return Err("Vote already resolved".into());
    }
    submit_vote(record, vote);
    let passed = vote_passed(record, state.governance_policy.vote_threshold);
    Ok(passed)
}

pub fn resolve_vote_if_passed(state: &mut CompanyState, vote_id: &str) -> Result<bool, String> {
    let (role, holder, should_resolve) = {
        let record = state.votes.get_mut(vote_id).ok_or("Vote not found")?;
        if record.resolved {
            return Ok(true);
        }
        let passed = vote_passed(record, state.governance_policy.vote_threshold);
        (record.target_role, record.target_holder.clone(), passed)
    };

    if should_resolve {
        remove_holder_from_role(state, role, &holder);
        if let Some(record) = state.votes.get_mut(vote_id) {
            record.resolved = true;
        }
        return Ok(true);
    }
    Ok(false)
}

pub fn auto_trigger_value_drop_vote(
    state: &mut CompanyState,
    current_price: f64,
    now: DateTime<Utc>,
    vote_id: &str,
    role: RoleTier,
    holder: &str,
) -> bool {
    if value_drop_triggered(state, current_price) {
        if !state.votes.contains_key(vote_id) {
            create_vote(state, vote_id, role, holder, "value_drop_trigger", now);
        }
        return true;
    }
    false
}

pub fn allocation_template(name: &str) -> Option<Vec<WorkAllocation>> {
    match name {
        "marketing" => Some(vec![
            WorkAllocation { holder_id: "lead".into(), weight: 5.0 },
            WorkAllocation { holder_id: "writer".into(), weight: 3.0 },
            WorkAllocation { holder_id: "designer".into(), weight: 2.0 },
        ]),
        "engineering" => Some(vec![
            WorkAllocation { holder_id: "lead".into(), weight: 4.0 },
            WorkAllocation { holder_id: "backend".into(), weight: 3.0 },
            WorkAllocation { holder_id: "frontend".into(), weight: 2.0 },
            WorkAllocation { holder_id: "qa".into(), weight: 1.0 },
        ]),
        "ops" => Some(vec![
            WorkAllocation { holder_id: "ops_lead".into(), weight: 6.0 },
            WorkAllocation { holder_id: "finance".into(), weight: 2.0 },
            WorkAllocation { holder_id: "admin".into(), weight: 2.0 },
        ]),
        _ => None,
    }
}

pub fn pm_create_task(
    state: &mut CompanyState,
    id: &str,
    title: &str,
    summary: &str,
    dod: Vec<String>,
    deliverables: Vec<Deliverable>,
    max_total_loc: usize,
) {
    let now = Utc::now();
    state.tasks.insert(
        id.to_string(),
        Task {
            id: id.to_string(),
            title: title.to_string(),
            summary: summary.to_string(),
            definition_of_done: dod,
            deliverables,
            status: TaskStatus::Draft,
            created_at: now,
            updated_at: now,
            assigned: vec![],
            max_total_loc,
            require_summary: true,
            require_tests: true,
        },
    );
}

pub fn pm_ready_task(state: &mut CompanyState, id: &str) -> Result<(), String> {
    let task = state.tasks.get_mut(id).ok_or("Task not found")?;
    if task.definition_of_done.is_empty() {
        return Err("Definition of done required".into());
    }
    if task.deliverables.is_empty() {
        return Err("At least one deliverable required".into());
    }
    task.status = TaskStatus::Ready;
    task.updated_at = Utc::now();
    Ok(())
}

pub fn pm_assign_task(state: &mut CompanyState, id: &str, assignee_id: &str, role: &str) -> Result<(), String> {
    let task = state.tasks.get_mut(id).ok_or("Task not found")?;
    task.assigned.push(TaskAssignment {
        assignee_id: assignee_id.to_string(),
        role: role.to_string(),
        status: TaskStatus::InProgress,
    });
    task.status = TaskStatus::InProgress;
    task.updated_at = Utc::now();
    Ok(())
}

pub fn pm_submit_for_review(state: &mut CompanyState, id: &str) -> Result<(), String> {
    let task = state.tasks.get_mut(id).ok_or("Task not found")?;
    task.status = TaskStatus::Review;
    task.updated_at = Utc::now();
    Ok(())
}

pub fn pm_finalize_task(state: &mut CompanyState, id: &str, loc_changed: usize, tests_run: bool) -> Result<(), String> {
    let task = state.tasks.get_mut(id).ok_or("Task not found")?;
    if loc_changed > task.max_total_loc {
        return Err("Code bloat guardrail: LOC limit exceeded".into());
    }
    if task.require_tests && !tests_run {
        return Err("Tests required before finalization".into());
    }
    task.status = TaskStatus::Done;
    task.updated_at = Utc::now();
    Ok(())
}

pub fn ensure_holder(state: &mut CompanyState, id: &str, name: &str) {
    state.holders.entry(id.to_string()).or_insert(Holder {
        id: id.to_string(),
        display_name: name.to_string(),
        tokens: 0.0,
        cash: 0.0,
        positions: vec![],
    });
}

pub fn assign_role_to_holder(state: &mut CompanyState, role: RoleTier, holder_id: &str) -> Result<(), String> {
    let holder = state.holders.get_mut(holder_id).ok_or("Holder not found")?;
    if !holder.positions.contains(&role) {
        holder.positions.push(role);
    }

    if role == RoleTier::Employee {
        return Ok(());
    }

    if let Some(pos) = state.positions.iter_mut().find(|p| p.tier == role && p.holder_id.is_none()) {
        pos.holder_id = Some(holder_id.to_string());
        pos.acquired_at = Some(Utc::now());
        pos.price_paid = Some(0.0);
        return Ok(());
    }

    // For fixed/special roles, create a position if missing
    let special = matches!(role, RoleTier::CEO | RoleTier::President | RoleTier::CoPresident | RoleTier::BoardSeat);
    if special {
        state.positions.push(RolePosition {
            tier: role,
            holder_id: Some(holder_id.to_string()),
            acquired_at: Some(Utc::now()),
            price_paid: Some(0.0),
        });
        return Ok(());
    }

    Err("No available position for role".into())
}

pub fn seed_roles(state: &mut CompanyState, ceo_id: &str, ceo_name: &str) -> Result<(), String> {
    ensure_holder(state, ceo_id, ceo_name);
    ensure_positions(state);

    assign_role_to_holder(state, RoleTier::CEO, ceo_id)?;
    assign_role_to_holder(state, RoleTier::BoardSeat, ceo_id)?;

    let defaults = vec![
        ("president", "President", RoleTier::President),
        ("co_president", "Co-President", RoleTier::CoPresident),
        ("csuite", "C-Suite", RoleTier::CSuite),
        ("director", "Director", RoleTier::Director),
        ("senior_manager", "Senior Manager", RoleTier::SeniorManager),
        ("manager", "Manager", RoleTier::Manager),
        ("employee", "Employee", RoleTier::Employee),
    ];

    for (id, name, role) in defaults {
        ensure_holder(state, id, name);
        let _ = assign_role_to_holder(state, role, id);
    }
    Ok(())
}

pub fn set_tokenomics(state: &mut CompanyState, total_supply_cap: f64, minted_supply: f64, allocations: Vec<TokenAllocation>) {
    state.tokenomics = Some(Tokenomics {
        total_supply_cap,
        minted_supply,
        allocations,
    });
}

pub fn tokenomics_report(state: &CompanyState) -> Option<serde_json::Value> {
    let t = state.tokenomics.as_ref()?;
    let allocated_total: f64 = t.allocations.iter().map(|a| a.percent).sum();
    Some(serde_json::json!({
        "total_supply_cap": t.total_supply_cap,
        "minted_supply": t.minted_supply,
        "remaining_supply": (t.total_supply_cap - t.minted_supply).max(0.0),
        "allocations": t.allocations,
        "allocations_total_percent": allocated_total
    }))
}

pub fn grant_tokens(state: &mut CompanyState, holder_id: &str, amount: f64) -> Result<(), String> {
    let holder = state.holders.get_mut(holder_id).ok_or("Holder not found")?;
    holder.tokens += amount;
    Ok(())
}

pub fn auto_onboard(
    state: &mut CompanyState,
    id: &str,
    name: &str,
    cash: f64,
) -> Result<bool, String> {
    onboard_holder(state, id, name, cash)?;
    state.onboarding_count += 1;

    let mut rewarded = false;
    if state.onboarding_count <= state.onboarding_policy.early_joiner_limit
        && state.onboarding_policy.early_joiner_reward > 0.0
    {
        grant_tokens(state, id, state.onboarding_policy.early_joiner_reward)?;
        rewarded = true;
    }
    Ok(rewarded)
}
