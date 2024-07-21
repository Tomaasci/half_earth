use crate::{
    events::{Effect, Probability},
    flavor::ProjectFlavor,
    kinds::{Output, OutputMap},
    npcs::{NPCRelation, NPC},
    Collection,
    HasId,
    Id,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::{
    Display,
    EnumDiscriminants,
    EnumIter,
    EnumString,
    IntoStaticStr,
};

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Default,
)]
pub enum Status {
    #[default]
    Inactive,
    Building,
    Active,
    Halted,
    Stalled,
    Finished,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Default,
    IntoStaticStr,
    EnumIter,
    EnumString,
    Display,
)]
pub enum Group {
    #[default]
    Other,
    Space,
    Nuclear,
    Restoration,
    Agriculture,
    Food,
    Geoengineering,
    Population,
    Control,
    Protection,
    Electrification,
    Behavior,
    Limits,
    Energy,
    Materials,
    Buildings,
    Cities,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Default,
    EnumIter,
    EnumString,
    IntoStaticStr,
    Display,
)]
pub enum Type {
    #[default]
    Policy,
    Research,
    Initiative,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Cost {
    Fixed(usize),
    Dynamic(f32, Factor),
}
impl Default for Cost {
    fn default() -> Self {
        Cost::Fixed(0)
    }
}

#[derive(
    Serialize,
    Deserialize,
    Copy,
    Clone,
    PartialEq,
    Debug,
    EnumDiscriminants,
)]
#[strum_discriminants(derive(
    EnumIter,
    EnumString,
    IntoStaticStr,
    Display
))]
#[strum_discriminants(name(FactorKind))]
pub enum Factor {
    Time,
    Income,
    Output(Output),
}

impl From<FactorKind> for Factor {
    fn from(kind: FactorKind) -> Self {
        match kind {
            FactorKind::Time => Factor::Time,
            FactorKind::Income => Factor::Income,
            FactorKind::Output => {
                Factor::Output(Output::default())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Outcome {
    pub effects: Vec<Effect>,
    pub probability: Probability,
}

#[derive(
    Debug, Deserialize, Serialize, Default, Clone, PartialEq,
)]
pub struct Upgrade {
    pub cost: usize,
    pub effects: Vec<Effect>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: Id,
    pub name: String,
    pub kind: Type,
    pub group: Group,
    pub ongoing: bool,
    pub gradual: bool,
    pub locked: bool,

    // For policies, the cost is the political capital cost;
    // for research and initiatives, it's the base years to completion
    pub cost: usize,
    pub base_cost: Cost,
    pub cost_modifier: f32,
    pub progress: f32,
    pub points: usize,
    pub estimate: usize,
    pub status: Status,
    pub level: usize,
    pub completed_at: usize,
    pub required_majority: f32,
    pub effects: Vec<Effect>,
    pub outcomes: Vec<Outcome>,
    pub upgrades: Vec<Upgrade>,
    pub active_outcome: Option<usize>,

    pub supporters: Vec<Id>,
    pub opposers: Vec<Id>,

    pub flavor: ProjectFlavor,
}

impl Display for Project {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl HasId for Project {
    fn id(&self) -> &Id {
        &self.id
    }
}

/// How many years a project takes to complete
/// for the given amount of points.
/// Has to be at least 1
pub fn years_for_points(points: usize, cost: usize) -> f32 {
    (cost as f32 / (points as f32).powf(1. / 2.75))
        .round()
        .max(1.)
}

impl Project {
    pub fn is_active(&self) -> bool {
        self.status == Status::Active
    }

    pub fn is_finished(&self) -> bool {
        self.status == Status::Finished
    }

    pub fn is_online(&self) -> bool {
        self.is_active() || self.is_finished()
    }

    pub fn is_building(&self) -> bool {
        self.status == Status::Building
    }

    pub fn is_haltable(&self) -> bool {
        self.is_online()
            && (self.kind == Type::Policy || self.ongoing)
    }

    pub fn can_downgrade(&self) -> bool {
        self.kind == Type::Policy && self.level > 0
    }

    /// Advance this project's implementation
    pub fn build(&mut self) -> bool {
        match &mut self.status {
            Status::Building => {
                self.progress += 1.
                    / years_for_points(self.points, self.cost);
                if self.progress >= 1. {
                    self.status = if self.ongoing {
                        Status::Active
                    } else {
                        Status::Finished
                    };
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn set_points(&mut self, points: usize) {
        self.points = points;
        self.estimate =
            years_for_points(self.points, self.cost) as usize;
    }

    pub fn update_cost(
        &mut self,
        year: usize,
        income_level: f32,
        demand: &OutputMap,
        modifier: f32,
    ) {
        let cost = match self.base_cost {
            Cost::Fixed(c) => c,
            Cost::Dynamic(m, factor) => {
                let c = match factor {
                    // Kind of arbitrarily choose 1980 as the starting point
                    Factor::Time => m * (year - 1980) as f32,
                    Factor::Income => m * (1. + income_level),
                    Factor::Output(output) => {
                        m * demand[output]
                    }
                };
                c.round() as usize
            }
        };
        self.cost =
            (cost as f32 * self.cost_modifier * modifier)
                .round() as usize;
    }

    pub fn upgrade(&mut self) -> bool {
        if self.level < self.upgrades.len() {
            self.level += 1;
            true
        } else {
            false
        }
    }

    pub fn downgrade(&mut self) -> bool {
        if self.level > 0 {
            self.level -= 1;
            true
        } else {
            false
        }
    }

    pub fn next_upgrade(&self) -> Option<&Upgrade> {
        self.upgrades.get(self.level)
    }

    pub fn prev_upgrade(&self) -> Option<&Upgrade> {
        if self.level > 0 {
            self.upgrades.get(self.level - 1)
        } else {
            None
        }
    }

    pub fn active_effects(&self) -> &Vec<Effect> {
        if self.level == 0 {
            &self.effects
        } else {
            &self.upgrades[self.level - 1].effects
        }
    }

    pub fn active_effects_with_outcomes(&self) -> Vec<&Effect> {
        let mut effects = vec![];
        if self.is_online() {
            effects.extend(self.active_effects().iter());
            if let Some(id) = self.active_outcome {
                effects
                    .extend(self.outcomes[id].effects.iter());
            }
        }
        effects
    }

    pub fn update_required_majority(
        &mut self,
        npcs: &Collection<NPC>,
    ) {
        let opposers = self
            .opposers
            .iter()
            .filter(|id| {
                !npcs[*id].locked
                    && npcs[*id].relation() != NPCRelation::Ally
            })
            .count();
        let supporters = self
            .supporters
            .iter()
            .filter(|id| !npcs[*id].locked)
            .count();
        self.required_majority =
            if opposers > supporters { 0.5 } else { 0. };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::events::{
        Comparator,
        Condition,
        Likelihood,
        WorldVariable,
    };
    use rand::SeedableRng;

    #[test]
    fn test_build_project() {
        let mut p = Project {
            id: "test_project",
            name: "Test Project",
            cost: 1,
            base_cost: Cost::Fixed(1),
            cost_modifier: 1.,
            required_majority: 0.,
            level: 0,
            ongoing: false,
            gradual: false,
            locked: false,
            kind: Type::Policy,
            group: Group::Other,
            status: Status::Building,
            progress: 0.,
            estimate: 0,
            points: 1,
            completed_at: 0,
            effects: vec![],
            upgrades: vec![],
            outcomes: vec![Outcome {
                effects: vec![],
                probability: Probability {
                    likelihood: Likelihood::Guaranteed,
                    conditions: vec![],
                },
            }],
            active_outcome: None,
            opposers: vec![],
            supporters: vec![],
        };

        for _ in 0..12 {
            p.build();
        }
        assert_eq!(p.status, Status::Finished);

        p.ongoing = true;
        p.status = Status::Building;
        p.progress = 0.;
        for _ in 0..12 {
            p.build();
        }
        assert_eq!(p.status, Status::Active);
    }

    #[test]
    fn test_project_estimate() {
        let mut p = Project {
            id: "test_project",
            name: "Test Project",
            cost: 10,
            base_cost: Cost::Fixed(10),
            cost_modifier: 1.,
            required_majority: 0.,
            level: 0,
            ongoing: false,
            gradual: false,
            locked: false,
            kind: Type::Policy,
            group: Group::Other,
            status: Status::Building,
            progress: 0.,
            estimate: 0,
            points: 0,
            completed_at: 0,
            effects: vec![],
            upgrades: vec![],
            outcomes: vec![Outcome {
                effects: vec![],
                probability: Probability {
                    likelihood: Likelihood::Guaranteed,
                    conditions: vec![],
                },
            }],
            active_outcome: None,
            opposers: vec![],
            supporters: vec![],
        };

        p.set_points(1);
        assert_eq!(p.estimate, 10);
        let prev_estimate = p.estimate;

        p.set_points(10);
        assert!(prev_estimate > p.estimate);
    }

    #[test]
    fn test_project_outcomes() {
        let mut rng: SmallRng = SeedableRng::seed_from_u64(0);
        let p = Project {
            id: "test_project",
            name: "Test Project",
            cost: 1,
            base_cost: Cost::Fixed(1),
            cost_modifier: 1.,
            required_majority: 0.,
            level: 0,
            ongoing: false,
            gradual: false,
            locked: false,
            kind: Type::Policy,
            group: Group::Other,
            status: Status::Building,
            progress: 0.,
            estimate: 0,
            points: 0,
            completed_at: 0,
            effects: vec![],
            upgrades: vec![],
            outcomes: vec![
                Outcome {
                    effects: vec![],
                    probability: Probability {
                        likelihood: Likelihood::Guaranteed,
                        conditions: vec![
                            Condition::WorldVariable(
                                WorldVariable::Year,
                                Comparator::Equal,
                                10.,
                            ),
                        ],
                    },
                },
                Outcome {
                    effects: vec![],
                    probability: Probability {
                        likelihood: Likelihood::Guaranteed,
                        conditions: vec![],
                    },
                },
            ],
            active_outcome: None,
            opposers: vec![],
            supporters: vec![],
        };

        let mut state = State::default();

        // Should be the second outcome
        // since the first condition isn't met
        let outcome = p.roll_outcome(&state, &mut rng);
        let (_outcome, i) = outcome.unwrap();
        assert_eq!(i, 1);

        // Now should be the first,
        state.world.year = 10;
        let outcome = p.roll_outcome(&state, &mut rng);
        let (_outcome, i) = outcome.unwrap();
        assert_eq!(i, 0);
    }
}
