use super::{PlayerVariable, WorldVariable};
use crate::{
    kinds::{Byproduct, Feedstock, Output, Resource},
    production::ProcessFeature,
    regions::{Latitude, Region},
    state::State,
};
use serde::{Deserialize, Serialize};
use std::ops::Mul;
use strum::{EnumDiscriminants, IntoStaticStr};

const MIGRATION_WAVE_PERCENT_POP: f32 = 0.1;
const CLOSED_BORDERS_MULTILPIER: f32 = 0.5;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Request {
    Project,
    Process,
}

#[derive(
    Serialize, Deserialize, PartialEq, Debug, Clone, Copy,
)]
pub enum Flag {
    RepeatTutorial,
    SkipTutorial,
    Electrified,
    Vegetarian,
    Vegan,
    ClosedBorders,
    HyperResearch,
    StopDevelopment,
    FastDevelopment,
    Degrowth,
    MetalsShortage,
    DeepSeaMining,
    ParliamentSuspended,
    MoreLabor,
    MoreAutomation,
    MoreLeisure,
    EcosystemModeling,
    LaborResistance,
    LaborSabotage,
    AlienEncounter,
    BailedOut,
}
impl std::fmt::Display for Flag {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let desc = match self {
          Self::HyperResearch => "Research points are cheaper.",
          Self::ClosedBorders => "Limits cross-region migration.",
          Self::AlienEncounter => "Encountered extraterrestrials",
          Self::ParliamentSuspended => "A parliamentary majority is no longer required for any project.",
          Self::Electrified => "80% of fuel demand becomes electricity demand.",
          Self::Vegan => "90% of animal calorie demand is met with plant calories.",
          Self::BailedOut => "You've been bailed out once",
          Self::FastDevelopment => "Underdeveloped regions develop more quickly.",
          Self::Degrowth => "Wealthy regions income levels and consumption will decline.",
          Self::MetalsShortage => "Infrastructure projects take 20% longer to finish.",
          Self::MoreLabor => "Projects take less time to complete.",
          Self::LaborResistance => "Projects take longer to complete.",
          Self::MoreLeisure => "Projects take more time to complete.",
          Self::DeepSeaMining => "Stops or prevents metals shortages.",
          Self::MoreAutomation => "Projects take less time to complete.",
          Self::Vegetarian => "75% of animal calorie demand is met with plant calories.",
          Self::StopDevelopment => "Stops regional development throughout the world.",
          Self::LaborSabotage => "Projects take longer to complete.",
          Self::EcosystemModeling => "Restoration projects take less time to complete.",
          Self::RepeatTutorial => "Repeat the tutorial.",
          Self::SkipTutorial => "Skip the tutorial.",
        };
        write!(f, "{}", desc)
    }
}

#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
    EnumDiscriminants,
)]
#[strum_discriminants(derive(IntoStaticStr))]
pub enum Effect {
    WorldVariable(WorldVariable, f32),
    PlayerVariable(PlayerVariable, f32),
    RegionHabitability(Latitude, f32),

    Resource(Resource, f32),
    Demand(Output, f32),
    Output(Output, f32),
    DemandAmount(Output, f32),
    OutputForFeature(ProcessFeature, f32),
    OutputForProcess(usize, f32),
    CO2ForFeature(ProcessFeature, f32),
    BiodiversityPressureForFeature(ProcessFeature, f32),
    ProcessLimit(usize, f32),
    Feedstock(Feedstock, f32),

    AddEvent(usize),
    TriggerEvent(usize, usize),
    LocksProject(usize),
    UnlocksProject(usize),
    UnlocksProcess(usize),
    UnlocksNPC(usize),

    ProjectRequest(usize, bool, usize),
    ProcessRequest(usize, bool, usize),

    Migration,
    RegionLeave,
    TerminationShock,
    AddRegionFlag(String),

    AddFlag(Flag),
    AutoClick(usize, f32),
    NPCRelationship(usize, f32),

    ModifyProcessByproducts(usize, Byproduct, f32),
    ModifyIndustryByproducts(usize, Byproduct, f32),
    ModifyIndustryResources(usize, Resource, f32),
    ModifyIndustryResourcesAmount(usize, Resource, f32),
    ModifyEventProbability(usize, f32),
    ModifyIndustryDemand(usize, f32),
    DemandOutlookChange(Output, f32),
    IncomeOutlookChange(f32),
    ProjectCostModifier(usize, f32),

    ProtectLand(f32),

    BailOut(usize),
    GameOver,
}
impl AsRef<Effect> for Effect {
    fn as_ref(&self) -> &Effect {
        self
    }
}

fn check_game_over(state: &mut State) {
    if !state.is_ally("The Authoritarian")
        && state.outlook() < 0.
    {
        state.game_over = true;
    }
}

impl Effect {
    /// For comparing if two effects are of the same "type"
    /// and thus may be alternatives to one another.
    pub fn fingerprint(&self) -> String {
        let discrim: EffectDiscriminants = self.into();
        let discrim: &'static str = discrim.into();
        let subkind: &'static str = match self {
            Self::WorldVariable(var, _) => var.into(),
            Self::PlayerVariable(var, _) => var.into(),
            Self::RegionHabitability(lat, _) => lat.into(),
            Self::Resource(res, _) => res.into(),
            Self::Demand(out, _) => out.into(),
            Self::Output(out, _) => out.into(),
            Self::DemandAmount(out, _) => out.into(),
            Self::OutputForFeature(feat, _) => feat.into(),
            Self::CO2ForFeature(feat, _) => feat.into(),
            Self::BiodiversityPressureForFeature(feat, _) => {
                feat.into()
            }
            Self::Feedstock(fs, _) => fs.into(),
            Self::ModifyProcessByproducts(_, byp, _) => {
                byp.into()
            }
            Self::ModifyIndustryByproducts(_, byp, _) => {
                byp.into()
            }
            Self::ModifyIndustryResources(_, res, _) => {
                res.into()
            }
            Self::ModifyIndustryResourcesAmount(_, res, _) => {
                res.into()
            }
            Self::DemandOutlookChange(out, _) => out.into(),
            _ => "",
        };
        format!("{discrim}:{subkind}")
    }

    pub fn apply(
        &self,
        state: &mut State,
        region_id: Option<usize>,
    ) {
        match self {
            Effect::GameOver => {
                state.game_over = true;
            }
            Effect::BailOut(amount) => {
                if state.political_capital < 0 {
                    state.political_capital = 0;
                }
                state.political_capital += *amount as isize;
            }
            Effect::WorldVariable(var, change) => {
                match var {
                    WorldVariable::Year => {
                        state.world.year += *change as usize
                    }
                    WorldVariable::Population => {
                        state.world.change_population(*change)
                    }
                    WorldVariable::PopulationGrowth => {
                        state.population_growth_modifier +=
                            *change / 100.
                    }
                    WorldVariable::Emissions => {
                        state.byproduct_mods.co2 +=
                            *change * 1e15; // effect in Gt
                        state.co2_emissions += *change * 1e15; // Apply immediately
                    }
                    WorldVariable::ExtinctionRate => {
                        state.byproduct_mods.biodiversity -=
                            *change
                    }
                    WorldVariable::Outlook => {
                        state.world.base_outlook += *change;
                        check_game_over(state);
                    }
                    WorldVariable::Temperature => {
                        state.temperature_modifier += *change
                    }
                    WorldVariable::WaterStress => {
                        state.water_stress += *change
                    }
                    WorldVariable::SeaLevelRise => {
                        state.world.sea_level_rise += *change
                    }
                    WorldVariable::SeaLevelRiseRate => {
                        state.sea_level_rise_modifier += *change
                    }
                    WorldVariable::Precipitation => {
                        state.precipitation += *change
                    }
                }
            }
            Effect::PlayerVariable(var, change) => {
                match var {
                    PlayerVariable::PoliticalCapital => {
                        state.political_capital +=
                            *change as isize
                    }
                    PlayerVariable::ResearchPoints => {
                        state.research_points +=
                            *change as isize
                    } // TODO need to use the rust state for points then
                    _ => (),
                }
            }
            Effect::RegionHabitability(latitude, change) => {
                for region in state
                    .world
                    .regions
                    .iter_mut()
                    .filter(|r| &r.latitude == latitude)
                {
                    region.base_habitability += change;
                }
            }
            Effect::Resource(resource, amount) => {
                state.resources[*resource] += amount;
            }
            Effect::Demand(output, pct_change) => {
                state.output_demand_modifier[*output] +=
                    pct_change;
            }
            Effect::DemandAmount(output, amount) => {
                state.output_demand_extras[*output] += amount;
            }
            Effect::Output(output, pct_change) => {
                state.output_modifier[*output] += pct_change;
            }
            Effect::OutputForFeature(feat, pct_change) => {
                for process in state
                    .world
                    .processes
                    .iter_mut()
                    .filter(|p| p.features.contains(feat))
                {
                    process.output_modifier += pct_change;
                }
            }
            Effect::OutputForProcess(id, pct_change) => {
                let process = &mut state.world.processes[*id];
                process.output_modifier += pct_change;
            }
            Effect::CO2ForFeature(feat, pct_change) => {
                for process in state
                    .world
                    .processes
                    .iter_mut()
                    .filter(|p| p.features.contains(feat))
                {
                    process.byproduct_modifiers.co2 +=
                        pct_change;
                }
            }
            Effect::BiodiversityPressureForFeature(
                feat,
                pct_change,
            ) => {
                for process in state
                    .world
                    .processes
                    .iter_mut()
                    .filter(|p| p.features.contains(feat))
                {
                    process.byproduct_modifiers.biodiversity +=
                        pct_change;
                }
            }
            Effect::ProcessLimit(id, change) => {
                let process = &mut state.world.processes[*id];
                if let Some(limit) = process.limit {
                    process.limit = Some(limit + change);
                }
            }
            Effect::Feedstock(feedstock, pct_change) => {
                state.feedstocks[*feedstock] *= 1. + pct_change;
            }
            Effect::AddEvent(id) => {
                state.event_pool.events[*id].locked = false;
            }
            Effect::TriggerEvent(id, years) => {
                state
                    .event_pool
                    .queue_event(*id, region_id, *years);
            }
            Effect::LocksProject(id) => {
                state.world.projects[*id].locked = true;
            }
            Effect::UnlocksProject(id) => {
                state.world.projects[*id].locked = false;
            }
            Effect::UnlocksProcess(id) => {
                state.world.processes[*id].locked = false;
            }
            Effect::UnlocksNPC(id) => {
                state.npcs[*id].locked = false;
            }
            Effect::ProjectRequest(id, active, bounty) => {
                state.requests.push((
                    Request::Project,
                    *id,
                    *active,
                    *bounty,
                ));
            }
            Effect::ProcessRequest(id, active, bounty) => {
                state.requests.push((
                    Request::Process,
                    *id,
                    *active,
                    *bounty,
                ));
            }
            Effect::Migration => {
                if let Some(id) = region_id {
                    let modifier = if state
                        .flags
                        .contains(&Flag::ClosedBorders)
                    {
                        CLOSED_BORDERS_MULTILPIER
                    } else {
                        1.
                    };
                    let leave_pop = state.world.regions[id]
                        .population
                        * MIGRATION_WAVE_PERCENT_POP
                        * modifier;
                    state.world.regions[id].population -=
                        leave_pop;

                    // Find the most habitable regions
                    let mean_habitability: f32 =
                        state.world.habitability();
                    let target_regions: Vec<&mut Region> =
                        state
                            .world
                            .regions
                            .iter_mut()
                            .filter(|r| {
                                r.id != id
                                    && r.habitability()
                                        > mean_habitability
                            })
                            .collect();
                    let per_region =
                        leave_pop / target_regions.len() as f32;
                    for region in target_regions {
                        region.population += per_region;
                    }
                }
            }
            Effect::RegionLeave => {
                if let Some(id) = region_id {
                    state.world.regions[id].seceded = true;
                }
            }
            Effect::AddRegionFlag(flag) => {
                if let Some(id) = region_id {
                    state.world.regions[id]
                        .flags
                        .push(flag.to_string());
                }
            }
            Effect::AddFlag(flag) => {
                state.flags.push(*flag);
            }
            Effect::NPCRelationship(id, change) => {
                state.npcs[*id].relationship += change;
            }

            Effect::ModifyProcessByproducts(
                id,
                byproduct,
                change,
            ) => {
                state.world.processes[*id]
                    .byproduct_modifiers[*byproduct] += change;
            }
            Effect::ModifyIndustryByproducts(
                id,
                byproduct,
                change,
            ) => {
                state.world.industries[*id]
                    .byproduct_modifiers[*byproduct] += change;
            }
            Effect::ModifyIndustryResources(
                id,
                resource,
                change,
            ) => {
                state.world.industries[*id]
                    .resource_modifiers[*resource] += change;
            }
            Effect::ModifyIndustryResourcesAmount(
                id,
                resource,
                change,
            ) => {
                state.world.industries[*id].resources
                    [*resource] += change;
            }
            Effect::ModifyEventProbability(id, change) => {
                state.event_pool.events[*id].prob_modifier +=
                    change;
            }
            Effect::ModifyIndustryDemand(id, change) => {
                state.world.industries[*id].demand_modifier +=
                    change;
            }
            Effect::DemandOutlookChange(output, mult) => {
                for region in &mut state.world.regions {
                    region.outlook += (mult
                        * region.demand_level(
                            output,
                            &state.world.output_demand,
                        ) as f32)
                        .floor();
                }
                check_game_over(state);
            }
            Effect::IncomeOutlookChange(mult) => {
                for region in &mut state.world.regions {
                    region.outlook += (mult
                        * region.income_level() as f32)
                        .floor();
                }
                check_game_over(state);
            }
            Effect::ProjectCostModifier(id, change) => {
                state.world.projects[*id].cost_modifier +=
                    change;
            }
            Effect::ProtectLand(percent) => {
                state.protected_land += percent / 100.;
            }

            // Effects like AutoClick have no impact in the engine side
            _ => (),
        }
    }

    pub fn unapply(
        &self,
        state: &mut State,
        region_id: Option<usize>,
    ) {
        match self {
            Effect::WorldVariable(var, change) => {
                match var {
                    WorldVariable::Year => {
                        state.world.year -= *change as usize
                    }
                    WorldVariable::Population => {
                        state.world.change_population(-*change)
                    }
                    WorldVariable::PopulationGrowth => {
                        state.population_growth_modifier -=
                            *change / 100.
                    }
                    WorldVariable::Emissions => {
                        state.byproduct_mods.co2 -=
                            *change * 1e15;
                        state.co2_emissions -= *change * 1e15; // Apply immediately
                    }
                    WorldVariable::ExtinctionRate => {
                        state.byproduct_mods.biodiversity +=
                            *change
                    }
                    WorldVariable::Outlook => {
                        state.world.base_outlook -= *change
                    }
                    WorldVariable::Temperature => {
                        state.temperature_modifier -= *change
                    }
                    WorldVariable::WaterStress => {
                        state.water_stress -= *change
                    }
                    WorldVariable::SeaLevelRise => {
                        state.world.sea_level_rise -= *change
                    }
                    WorldVariable::SeaLevelRiseRate => {
                        state.sea_level_rise_modifier -= *change
                    }
                    WorldVariable::Precipitation => {
                        state.precipitation -= *change
                    }
                }
            }
            Effect::PlayerVariable(var, change) => match var {
                PlayerVariable::PoliticalCapital => {
                    state.political_capital -= *change as isize
                }
                PlayerVariable::ResearchPoints => {
                    state.research_points -= *change as isize
                }
                _ => (),
            },
            Effect::RegionHabitability(latitude, change) => {
                for region in state
                    .world
                    .regions
                    .iter_mut()
                    .filter(|r| &r.latitude == latitude)
                {
                    region.base_habitability -= change;
                }
            }
            Effect::Resource(resource, amount) => {
                state.resources[*resource] -= amount;
            }
            Effect::Demand(output, pct_change) => {
                state.output_demand_modifier[*output] -=
                    pct_change;
            }
            Effect::DemandAmount(output, amount) => {
                state.output_demand_extras[*output] -= amount;
            }
            Effect::Output(output, pct_change) => {
                state.output_modifier[*output] -= pct_change;
            }
            Effect::OutputForFeature(feat, pct_change) => {
                for process in state
                    .world
                    .processes
                    .iter_mut()
                    .filter(|p| p.features.contains(feat))
                {
                    process.output_modifier -= pct_change;
                }
            }
            Effect::OutputForProcess(id, pct_change) => {
                let process = &mut state.world.processes[*id];
                process.output_modifier -= pct_change;
            }
            Effect::CO2ForFeature(feat, pct_change) => {
                for process in state
                    .world
                    .processes
                    .iter_mut()
                    .filter(|p| p.features.contains(feat))
                {
                    process.byproduct_modifiers.co2 -=
                        pct_change;
                }
            }
            Effect::BiodiversityPressureForFeature(
                feat,
                pct_change,
            ) => {
                for process in state
                    .world
                    .processes
                    .iter_mut()
                    .filter(|p| p.features.contains(feat))
                {
                    process.byproduct_modifiers.biodiversity -=
                        pct_change;
                }
            }
            Effect::ProcessLimit(id, change) => {
                let process = &mut state.world.processes[*id];
                if let Some(limit) = process.limit {
                    process.limit = Some(limit - change);
                }
            }
            Effect::Feedstock(feedstock, pct_change) => {
                state.feedstocks[*feedstock] /= 1. + pct_change;
            }
            Effect::NPCRelationship(id, change) => {
                state.npcs[*id].relationship -= change;
            }
            Effect::ModifyProcessByproducts(
                id,
                byproduct,
                change,
            ) => {
                state.world.processes[*id]
                    .byproduct_modifiers[*byproduct] -= change;
            }
            Effect::ModifyIndustryByproducts(
                id,
                byproduct,
                change,
            ) => {
                state.world.industries[*id]
                    .byproduct_modifiers[*byproduct] -= change;
            }
            Effect::ModifyIndustryResources(
                id,
                resource,
                change,
            ) => {
                state.world.industries[*id]
                    .resource_modifiers[*resource] -= change;
            }
            Effect::ModifyIndustryResourcesAmount(
                id,
                resource,
                change,
            ) => {
                state.world.industries[*id].resources
                    [*resource] -= change;
            }
            Effect::ModifyEventProbability(id, change) => {
                state.event_pool.events[*id].prob_modifier -=
                    change;
            }
            Effect::ModifyIndustryDemand(id, change) => {
                state.world.industries[*id].demand_modifier -=
                    change;
            }
            Effect::DemandOutlookChange(output, mult) => {
                for region in &mut state.world.regions {
                    region.outlook -= (mult
                        * region.demand_level(
                            output,
                            &state.world.output_demand,
                        ) as f32)
                        .floor();
                }
            }
            Effect::IncomeOutlookChange(mult) => {
                for region in &mut state.world.regions {
                    region.outlook -= (mult
                        * region.income_level() as f32)
                        .floor();
                }
            }
            Effect::ProjectCostModifier(id, change) => {
                state.world.projects[*id].cost_modifier -=
                    change;
            }
            Effect::TerminationShock => {
                let p = state
                    .world
                    .projects
                    .iter()
                    .find(|p| {
                        p.name == "Solar Radiation Management"
                    })
                    .unwrap();
                let effects = p.active_effects();
                let mut temp = 0.;
                for eff in effects {
                    match eff {
                        Effect::WorldVariable(typ, val) => {
                            match typ {
                                WorldVariable::Temperature => {
                                    temp += val
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    };
                }
                state.temperature_modifier -= temp;
            }
            Effect::ProtectLand(percent) => {
                state.protected_land -= percent / 100.;
            }
            Effect::AddFlag(flag) => {
                if let Some(idx) =
                    state.flags.iter().position(|x| x == flag)
                {
                    state.flags.remove(idx);
                }
            }
            Effect::LocksProject(id) => {
                state.world.projects[*id].locked = false;
            }
            Effect::UnlocksProject(id) => {
                state.world.projects[*id].locked = true;
            }
            Effect::UnlocksProcess(id) => {
                state.world.processes[*id].locked = true;
            }
            Effect::UnlocksNPC(id) => {
                state.npcs[*id].locked = true;
            }

            // Other effects aren't reversible
            _ => (),
        }
    }
}

// For scaling effects by float
impl Mul<f32> for Effect {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        match self {
            Effect::WorldVariable(var, val) => {
                Effect::WorldVariable(var, val * rhs)
            }
            Effect::PlayerVariable(var, val) => {
                Effect::PlayerVariable(var, val * rhs)
            }
            Effect::Resource(resource, val) => {
                Effect::Resource(resource, val * rhs)
            }
            Effect::Demand(output, val) => {
                Effect::Demand(output, val * rhs)
            }
            Effect::Output(output, val) => {
                Effect::Output(output, val * rhs)
            }
            Effect::DemandAmount(output, val) => {
                Effect::DemandAmount(output, val * rhs)
            }
            Effect::OutputForFeature(feat, val) => {
                Effect::OutputForFeature(feat, val * rhs)
            }
            Effect::OutputForProcess(id, val) => {
                Effect::OutputForProcess(id, val * rhs)
            }
            Effect::Feedstock(feedstock, val) => {
                Effect::Feedstock(feedstock, val * rhs)
            }
            Effect::ModifyIndustryByproducts(
                id,
                byproduct,
                val,
            ) => Effect::ModifyIndustryByproducts(
                id,
                byproduct,
                val * rhs,
            ),
            Effect::ModifyIndustryResources(
                id,
                resource,
                val,
            ) => Effect::ModifyIndustryResources(
                id,
                resource,
                val * rhs,
            ),
            Effect::ModifyIndustryResourcesAmount(
                id,
                resource,
                val,
            ) => Effect::ModifyIndustryResources(
                id,
                resource,
                val * rhs,
            ),
            Effect::ModifyIndustryDemand(id, val) => {
                Effect::ModifyIndustryDemand(id, val * rhs)
            }
            Effect::ModifyEventProbability(id, val) => {
                Effect::ModifyEventProbability(id, val * rhs)
            }
            Effect::DemandOutlookChange(output, val) => {
                Effect::DemandOutlookChange(output, val * rhs)
            }
            Effect::IncomeOutlookChange(val) => {
                Effect::IncomeOutlookChange(val * rhs)
            }
            Effect::ProjectCostModifier(id, val) => {
                Effect::ProjectCostModifier(id, val * rhs)
            }
            Effect::ProtectLand(val) => {
                Effect::ProtectLand(val * rhs)
            }
            _ => self,
        }
    }
}

pub fn mean_income_outlook_change(
    mult: f32,
    state: &State,
) -> f32 {
    state
        .world
        .regions
        .iter()
        .map(|region| {
            (mult * region.income_level() as f32).floor()
        })
        .sum::<f32>()
        / state.world.regions.len() as f32
}

pub fn mean_demand_outlook_change(
    mult: f32,
    output: &Output,
    state: &State,
) -> f32 {
    state
        .world
        .regions
        .iter()
        .map(|region| {
            (mult
                * region.demand_level(
                    output,
                    &state.world.output_demand,
                ) as f32)
                .floor()
        })
        .sum::<f32>()
        / state.world.regions.len() as f32
}
