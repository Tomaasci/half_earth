use crate::{vars::Var, views::Factor};
use enum_iterator::Sequence;
use enum_map::EnumMap;
use hes_engine::{
    events::IconEvent,
    game::Update,
    kinds::Output,
    regions::Income,
    state::State,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The state at the start of a 5-year cycle,
/// for generating comparisons for the report.
#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleStart {
    pub year: usize,
    pub emissions: f32,
    pub extinction_rate: f32,
    pub contentedness: f32,
    pub temperature: f32,
    pub region_incomes: Vec<Income>,

    // Seats in parliament for each NPC faction
    pub parliament: Vec<f32>,
    pub completed_projects: Vec<usize>,
}

/// Currently staged plan changes.
#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanChange {
    pub points: usize,
    pub upgrades: usize,
    pub downgrades: usize,
    pub withdrawn: bool,
    pub passed: bool,
}

/// Available/unused points.
#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Points {
    pub research: isize,
    pub initiative: isize,
    pub refundable_research: usize,
}

/// Phase of the game.
#[derive(
    Default, Clone, Copy, PartialEq, Serialize, Deserialize,
)]
pub enum Phase {
    #[default]
    Intro,
    Interstitial,
    Planning,
    Events,
    Report,
    GameOver,
    GameWin,
}

#[derive(
    Default,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    Sequence,
)]
pub enum Tutorial {
    #[default]
    Projects,
    ProjectsBack,
    Processes,
    ProcessesBack,
    Parliament,
    Dashboard,
    Regions,
    Plan,
    Ready,
}
impl Tutorial {
    pub fn advance(&mut self) {
        if let Some(next) = self.next() {
            *self = next;
        }
    }
}

/// Transient UI-state that is not preserved b/w sessions.
#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct UIState {
    pub phase: Phase,
    pub start_year: usize,
    new_run_count: u32,
    tutorial_restarted: bool,
    pub tutorial: Tutorial,
    pub factors: EnumMap<Var, Vec<Factor>>,

    pub annual_region_events: HashMap<usize, Vec<IconEvent>>,
    pub world_events: Vec<usize>,

    /// Emissions are three-tuples of `(CO2, CH4, N2O)`.
    pub past_emissions: Vec<(f64, f64, f64)>,

    // Track which events have occurred TODO
    // pub events: Vec<>,

    // // Track planned process mix changes
    pub process_mix_changes:
        EnumMap<Output, HashMap<usize, isize>>,

    // // Track changes made to the plan
    // // in a given session, so they can
    // // be reversed/refunded
    pub plan_changes: HashMap<usize, PlanChange>,
    pub queued_upgrades: HashMap<usize, bool>,
    //
    // Compare beginning and end
    pub cycle_start_state: CycleStart,

    pub points: Points,

    /// Viewed project and process ids,
    /// so we can keep track of which ones are new
    pub viewed: Vec<String>,
}
impl UIState {
    pub fn cycle_start_snapshot(&mut self, state: &State) {
        self.annual_region_events.clear();
        self.world_events.clear();

        self.cycle_start_state.year = state.world.year;
        self.cycle_start_state.extinction_rate =
            state.world.extinction_rate;
        self.cycle_start_state.contentedness = state.outlook();
        self.cycle_start_state.temperature =
            state.world.temperature;
        self.cycle_start_state.emissions = state.emissions_gt();
        self.cycle_start_state.region_incomes = state
            .world
            .regions
            .iter()
            .map(|r| r.income)
            .collect();
        self.cycle_start_state.parliament =
            state.npcs.iter().map(|npc| npc.seats).collect();
        self.cycle_start_state.completed_projects.clear();
    }
}
