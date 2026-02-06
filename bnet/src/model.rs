use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleTier {
    Employee,
    Manager,
    SeniorManager,
    Director,
    CSuite,
    President,
    CoPresident,
    CEO,
    BoardSeat,
}

impl fmt::Display for RoleTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RoleTier::Employee => "employee",
            RoleTier::Manager => "manager",
            RoleTier::SeniorManager => "senior_manager",
            RoleTier::Director => "director",
            RoleTier::CSuite => "csuite",
            RoleTier::President => "president",
            RoleTier::CoPresident => "co_president",
            RoleTier::CEO => "ceo",
            RoleTier::BoardSeat => "board_seat",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for RoleTier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "employee" => Ok(RoleTier::Employee),
            "manager" => Ok(RoleTier::Manager),
            "senior_manager" | "seniormanager" => Ok(RoleTier::SeniorManager),
            "director" => Ok(RoleTier::Director),
            "csuite" | "c_suite" => Ok(RoleTier::CSuite),
            "president" => Ok(RoleTier::President),
            "co_president" | "copresident" => Ok(RoleTier::CoPresident),
            "ceo" => Ok(RoleTier::CEO),
            "board_seat" | "board" => Ok(RoleTier::BoardSeat),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePosition {
    pub tier: RoleTier,
    pub holder_id: Option<String>,
    pub acquired_at: Option<DateTime<Utc>>,
    pub price_paid: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holder {
    pub id: String,
    pub display_name: String,
    pub tokens: f64,
    pub cash: f64,
    pub positions: Vec<RoleTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueEvent {
    pub timestamp: DateTime<Utc>,
    pub gross_revenue: f64,
    pub refund_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionPolicy {
    pub weekly_payout_percent: f64, // 20% of revenue
    pub halving_interval_days: i64, // every year
    pub genesis: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub vote_threshold: f64,   // 2/3
    pub value_drop_trigger: f64, // 20% drop
    pub value_window_days: i64, // 30-day average
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyState {
    pub holders: HashMap<String, Holder>,
    pub treasury_tokens: f64,
    pub treasury_cash: f64,
    pub positions: Vec<RolePosition>,
    pub employee_count: usize,
    pub token_price_history: Vec<(DateTime<Utc>, f64)>,
    pub emission_policy: EmissionPolicy,
    pub governance_policy: GovernancePolicy,
    pub votes: HashMap<String, VoteRecord>,
    pub marketplace: Vec<PositionListing>,
    pub tasks: HashMap<String, Task>,
    pub tokenomics: Option<Tokenomics>,
    pub onboarding_policy: OnboardingPolicy,
    pub onboarding_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionBid {
    pub bidder_id: String,
    pub target_role: RoleTier,
    pub bid_amount: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAllocation {
    pub holder_id: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter_id: String,
    pub weight: f64,
    pub approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRecord {
    pub id: String,
    pub target_role: RoleTier,
    pub target_holder: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub votes: Vec<Vote>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionListing {
    pub id: String,
    pub role: RoleTier,
    pub seller_id: String,
    pub price: f64,
    pub created_at: DateTime<Utc>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Draft,
    Ready,
    InProgress,
    Review,
    Done,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliverable {
    pub description: String,
    pub max_loc: usize,
    pub tests_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub assignee_id: String,
    pub role: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub definition_of_done: Vec<String>,
    pub deliverables: Vec<Deliverable>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assigned: Vec<TaskAssignment>,
    pub max_total_loc: usize,
    pub require_summary: bool,
    pub require_tests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAllocation {
    pub name: String,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokenomics {
    pub total_supply_cap: f64,
    pub minted_supply: f64,
    pub allocations: Vec<TokenAllocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingPolicy {
    pub early_joiner_limit: usize,
    pub early_joiner_reward: f64,
}
