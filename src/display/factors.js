import format from './format';
import state from '/src/state';
import display from './display';
import intensity from './intensity';
import {activeEffects} from './project';
import consts from '/src/consts.json';
import EVENTS from '/assets/content/events.json';

const VARS = ['land', 'water', 'energy', 'emissions', 'biodiversity', 'contentedness'];

function effectsFactor(k, effects) {
  if (k == 'emissions') {
    return effects.reduce((acc, eff) => {
      return acc + (eff.subtype == 'Emissions' ? eff.param : 0);
    }, 0);
  } else if (k == 'water') {
    // TODO no effects that influence this directly
    return 0;
  } else if (k == 'land') {
    return effects.reduce((acc, eff) => {
      return acc + (eff.type == 'ProtectLand' ? eff.param : 0);
    }, 0);
  } else if (k == 'energy') {
    // TODO no effects that influence this directly
    return 0;
  } else if (k == 'contentedness') {
    return effects.reduce((acc, eff) => {
      let amount = 0;
      if (eff.subtype == 'Outlook') {
        amount = eff.param;
      } else if (eff.type == 'IncomeOutlookChange') {
        // TODO
      } else if (eff.type == 'DemandOutlookChange') {
        // TODO
      }
      return acc + amount
    }, 0);
  } else if (k == 'biodiversity') {
    return effects.reduce((acc, eff) => {
      return acc + (eff.subtype == 'ExtinctionRate' ? eff.param : 0);
    }, 0);
  }
}

function projectFactors(k) {
  return state.gameState.projects.filter((p) => {
    return p.status == 'Active' || p.status == 'Finished';
  }).map((p) => {
    let effects = activeEffects(p);
    return {
      name: p.name,
      type: 'Project',
      amount: effectsFactor(k, effects)
    };
  }).filter((p) => p.amount !== 0);
}

function eventFactors(k) {
  return state.events.map(([eventId, _regionId]) => {
    let event = EVENTS[eventId];
    return {
      name: event.name,
      type: 'Event',
      amount: effectsFactor(k, event.effects)
    };
  }).filter((p) => p.amount !== 0);
}

function productionFactors(k) {
  let contributors = state.gameState.processes.map((p, i) => {
    return {
      demand: state.gameState.produced_by_process[i],
      ...p
    };
  }).concat(state.gameState.industries);

  return contributors.map((p) => {
    let base = 0;
    if (k == 'land' || k == 'water') {
      base = p.resources[k];
    } else if (k == 'energy') {
      base = (p.resources['electricity'] + p.resources['fuel']);
    } else if (k == 'emissions') {
      base = format.co2eq(p.byproducts);
    } else if (k == 'biodiversity') {
      base = (p.byproducts[k]/1e4 + p.resources['land']/consts.starting_resources.land) * 100;
    }

    let type =
      (p.output == 'Electricity' || p.output == 'Fuel')
      ? 'energy' : 'calories';

    let total = base * p.demand;
    let inten = intensity.intensity(base, k, type);

    let out = p.output ? display.enumKey(p.output) : null;
    return {
      name: p.name,
      produced: p.demand,
      output: out,
      intensity: inten,
      amount: total,
      displayAmount: format.formatResource[k](total),
      displayProduced: out != null ? format.output(p.demand, out) : null,
    }
  }).filter((p) => p.output != null || p.output == null && p.amount !== 0);
}

function rank() {
  let factors = {};
  VARS.forEach((k) => {
    let rankings = [];

    if (k !== 'contentedness') {
      rankings = rankings.concat(productionFactors(k));
    }
    rankings = rankings.concat(projectFactors(k));
    rankings = rankings.concat(eventFactors(k));

    if (k == 'contentedness') {
      if (state.gameState.world.temp_outlook !== 0) {
        rankings.push({
          type: 'Event',
          name: 'Temperature Change',
          amount: Math.round(state.gameState.world.temp_outlook)
        });
      }
    } else if (k == 'biodiversity') {
        rankings.push({
          type: 'Event',
          name: 'Sea Level Rise',
          amount: Math.round(state.gameState.world.sea_level_rise**2)
        });
        rankings.push({
          type: 'Event',
          name: 'Temperature Change',
          amount: Math.round(state.gameState.world.temperature**2)
        });
    }

    rankings.sort((a, b) => Math.abs(a.amount) > Math.abs(b.amount) ? -1 : 1)
    factors[k] = rankings;
  });

  return factors;
}

const tips = {
  emissions: (text, current) => {
    return {
      text,
      icon: 'emissions',
      card: {
        type: 'Factors',
        data: {
          icon: 'emissions',
          type: 'emissions',
          total: `${state.gameState.emissions.toFixed(1)}Gt`,
          current,
        }
      }
    }
  },
  biodiversity: (text, current) => {
    return {
      text,
      icon: 'extinction_rate',
      card: {
        type: 'Factors',
        data: {
          icon: 'extinction_rate',
          type: 'biodiversity',
          total: Math.round(state.gameState.world.extinction_rate),
          current,
        }
      }
    }
  },
  land: (text, current) => {
    return {
      text,
      icon: 'land',
      card: {
        type: 'Factors',
        data: {
          icon: 'land',
          type: 'land',
          total: `${Math.round(format.landUsePercent(state.gameState.resources_demand.land))}%`,
          current,
        }
      }
    }
  },
  energy: (text, current) => {
    let demand = state.gameState.output_demand;
    return {
      text,
      icon: 'energy',
      card: {
        type: 'Factors',
        data: {
          icon: 'energy',
          type: 'energy',
          total: `${((demand.electricity + demand.fuel) * 1e-9).toFixed(0)}TWh`,
          current,
        }
      }
    }
  },
  water: (text, current) => {
    return {
      text,
      icon: 'water',
      card: {
        type: 'Factors',
        data: {
          icon: 'water',
          type: 'water',
          total: `${Math.round(format.waterUsePercent(state.gameState.resources_demand.water))}%`,
          current,
        }
      }
    }
  },
  contentedness: (text, current) => {
    return {
      text,
      icon: 'contentedness',
      card: {
        type: 'Factors',
        data: {
          icon: 'contentedness',
          type: 'contentedness',
          total: Math.round(state.gameState.contentedness),
          current,
        }
      }
    }
  }
}


export default {rank, tips};
