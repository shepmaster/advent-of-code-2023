use petgraph::{
    algo::dominators,
    graphmap::DiGraphMap,
    visit::{self, Control},
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    iter, ops,
};

const INPUT: &str = include_str!("../input");

fn main() {
    let product = high_low_product(INPUT);
    // Part 1: 808146535
    println!("{product}");

    let presses = presses_until_rx_low(INPUT);
    // Part 2: 224370869958144 (too low)
    // -> multiplication of button presses, not cycle times
    //         224602953547789
    println!("{presses}")
}

fn high_low_product(s: &str) -> usize {
    let mut modules = Modules::new(s);

    let pulses = (0..1000).map(|_| modules.push_button()).sum::<Pulses>();

    pulses.product()
}

fn presses_until_rx_low(s: &str) -> usize {
    let modules = Modules::new(s);

    modules.graph()
}

#[derive(Debug, Clone)]
struct Modules<'a>(BTreeMap<&'a str, Module<'a>>);

impl<'a> Modules<'a> {
    fn new(s: &'a str) -> Modules<'a> {
        use Module::*;
        use Pulse::*;

        let mut modules = s
            .lines()
            .map(|line| {
                let (name, outputs) = line.split_once(" -> ").unwrap();

                let outputs = outputs.split(',').map(|o| o.trim()).collect();

                let module = if let Some(name) = name.strip_prefix('%') {
                    FlipFlop {
                        is_on: false,
                        name,
                        outputs,
                    }
                } else if let Some(name) = name.strip_prefix('&') {
                    Conjunction {
                        inputs: Default::default(),
                        name,
                        outputs,
                    }
                } else {
                    Plain { name, outputs }
                };

                (module.name(), module)
            })
            .collect::<BTreeMap<_, _>>();

        // Ensure all referenced outputs exist

        let all_outputs = modules
            .values()
            .flat_map(|modules| modules.outputs().iter().copied())
            .collect::<BTreeSet<_>>();

        for name in all_outputs {
            modules.entry(name).or_insert(Plain {
                name,
                outputs: vec![],
            });
        }

        // Backfill all conjunction incoming names

        let conjunctions = modules
            .values()
            .flat_map(|module| module.conjunction_name())
            .collect::<BTreeSet<_>>();

        let mut incoming = BTreeMap::new();

        for module in modules.values() {
            for &output in module.outputs() {
                if conjunctions.contains(output) {
                    incoming
                        .entry(output)
                        .or_insert_with(BTreeSet::new)
                        .insert(module.name());
                }
            }
        }

        for (name, incoming_names) in incoming {
            let module = modules
                .get_mut(name)
                .unwrap_or_else(|| panic!("Must have the module {name}"));

            match module {
                Conjunction { inputs, .. } => {
                    inputs.extend(incoming_names.into_iter().map(|n| (n, Low)));
                }
                _ => unreachable!("Must be a conjunction"),
            }
        }

        Modules(modules)
    }

    /// Idea: Find all subgraphs between the broadcaster and rx
    /// that have exactly one outgoing edge. These subgraphs can be
    /// cycle-reduced.
    ///
    /// Somehow, it turns out if we multiply all the cycle lengths
    /// together, that's the answer?
    fn graph(&self) -> usize {
        use petgraph::Direction::*;

        let mut g = DiGraphMap::new();

        for module in self.0.values() {
            let from = g.add_node(module.name());
            for to in module.outputs() {
                let to = g.add_node(to);

                g.add_edge(from, to, "");
            }
        }

        // let dot = petgraph::dot::Dot::new(&g).to_string();
        // std::fs::write("graph.dot", dot).unwrap();

        // Find all the subgraphs

        let b = g.add_node("broadcaster");
        let to = "rx";

        let mut subsets = vec![];

        for (_, from, _) in g.edges_directed(b, Outgoing) {
            let d = dominators::simple_fast(&g, from);
            if let Some(d) = d.dominators(to) {
                let to = d
                    .filter(|n| g.edges_directed(n, Incoming).count() == 1)
                    .nth(1); // skip rx itself

                if let Some(to) = to {
                    subsets.push((from, to));
                }
            }
        }

        // Find the cycle length of each subgraph

        subsets
            .into_iter()
            .map(|(from, to)| {
                // Find all nodes in this subgraph

                let mut subset_names = BTreeSet::new();

                visit::depth_first_search(&g, [from], |evt| match evt {
                    visit::DfsEvent::Discover(n, _) => {
                        subset_names.insert(n);

                        if n == to {
                            Control::<()>::Prune
                        } else {
                            Control::Continue
                        }
                    }
                    _ => Control::Continue,
                });

                let mut this = self.clone();

                // Keep only the steps relevant to this subgraph

                this.0.retain(|name, _| subset_names.contains(name));

                // Add back one the output node

                this.0.insert(
                    to,
                    Module::Plain {
                        name: to,
                        outputs: vec![],
                    },
                );

                this.detect_cycle("broadcaster", from, Pulse::Low)
            })
            .product()
    }

    fn state(&self) -> Vec<ModuleState> {
        self.0.values().map(|module| module.state()).collect()
    }

    fn detect_cycle(&mut self, from: &'a str, to: &'a str, pulse: Pulse) -> usize {
        let original_state = self.state();

        let mut length = 0;

        loop {
            self.push_button_core(from, to, pulse);

            length += 1;

            if original_state == self.state() {
                break length;
            }
        }
    }

    fn push_button(&mut self) -> Pulses {
        self.push_button_core("button", "broadcaster", Pulse::Low)
    }

    fn push_button_core(&mut self, from: &'a str, to: &'a str, pulse: Pulse) -> Pulses {
        use Module::*;
        use Pulse::*;

        let mut queue = VecDeque::from_iter([(from, to, pulse)]);
        let mut pulses = Pulses::default();

        while let Some((from, to, pulse)) = queue.pop_front() {
            //            eprintln!("{from} --{pulse:?}--> {to}");

            match pulse {
                Low => pulses.low += 1,
                High => pulses.high += 1,
            }

            let module = self
                .0
                .get_mut(to)
                .unwrap_or_else(|| panic!("Module {to} not found"));

            match module {
                FlipFlop { is_on, outputs, .. } => {
                    if pulse == Low {
                        *is_on = !*is_on;

                        let pulse = match is_on {
                            true => High,
                            false => Low,
                        };

                        queue.extend(outputs.iter().map(|&name| (to, name, pulse)));
                    }
                }

                Conjunction {
                    inputs, outputs, ..
                } => {
                    *inputs.entry(from).or_insert(Low) = pulse;

                    let all_high = inputs.values().all(|&p| p == High);
                    let pulse = match all_high {
                        true => Low,
                        false => High,
                    };
                    queue.extend(outputs.iter().map(|&name| (to, name, pulse)));
                }

                Plain { outputs, .. } => {
                    queue.extend(outputs.iter().map(|&name| (to, name, pulse)));
                }
            }
        }

        pulses
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ModuleState {
    FlipFlop { is_on: bool },

    Conjunction { inputs: Vec<Pulse> },

    Null,
}

#[derive(Debug, Clone)]
enum Module<'a> {
    FlipFlop {
        is_on: bool,
        name: &'a str,
        outputs: Vec<&'a str>,
    },
    Conjunction {
        inputs: BTreeMap<&'a str, Pulse>,
        name: &'a str,
        outputs: Vec<&'a str>,
    },
    Plain {
        name: &'a str,
        outputs: Vec<&'a str>,
    },
}

impl<'a> Module<'a> {
    fn name(&self) -> &'a str {
        use Module::*;

        match self {
            FlipFlop { name, .. } | Conjunction { name, .. } | Plain { name, .. } => name,
        }
    }

    fn conjunction_name(&self) -> Option<&'a str> {
        use Module::*;

        match self {
            FlipFlop { .. } => None,
            Conjunction { name, .. } => Some(*name),
            Plain { .. } => None,
        }
    }

    fn outputs(&self) -> &[&'a str] {
        use Module::*;

        match self {
            FlipFlop { outputs, .. } | Conjunction { outputs, .. } | Plain { outputs, .. } => {
                outputs
            }
        }
    }

    fn state(&self) -> ModuleState {
        use Module as M;
        use ModuleState as S;

        match *self {
            M::FlipFlop { is_on, .. } => S::FlipFlop { is_on },
            M::Conjunction { ref inputs, .. } => {
                let inputs = inputs.values().copied().collect();
                S::Conjunction { inputs }
            }
            M::Plain { .. } => S::Null,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Pulse {
    Low,
    High,
}

#[derive(Debug, Copy, Clone, Default)]
struct Pulses {
    low: usize,
    high: usize,
}

impl Pulses {
    fn product(&self) -> usize {
        let Self { low, high } = self;
        low * high
    }
}

impl ops::MulAssign<usize> for Pulses {
    fn mul_assign(&mut self, rhs: usize) {
        self.low *= rhs;
        self.high *= rhs;
    }
}

impl ops::AddAssign for Pulses {
    fn add_assign(&mut self, rhs: Self) {
        self.low += rhs.low;
        self.high += rhs.high;
    }
}

impl iter::Sum for Pulses {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut this = Self::default();
        for p in iter {
            this += p;
        }
        this
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");
    const EXAMPLE_INPUT_2: &str = include_str!("../example-input-2");

    #[test]
    fn example_1() {
        assert_eq!(32000000, high_low_product(EXAMPLE_INPUT_1));
    }

    #[test]
    fn example_2() {
        assert_eq!(11687500, high_low_product(EXAMPLE_INPUT_2));
    }
}
