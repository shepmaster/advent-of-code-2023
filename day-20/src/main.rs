use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    iter,
};

const INPUT: &str = include_str!("../input");

fn main() {
    let product = high_low_product(INPUT);
    // Part 1: 808146535
    println!("{product}");
}

fn high_low_product(s: &str) -> usize {
    let mut modules = Modules::new(s);

    let presses = (0..1000).map(|_| modules.push_button()).sum::<Presses>();

    presses.product()
}

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

        // ----------

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
        // ----------

        let conjunctions = modules
            .values()
            .flat_map(|module| match module {
                FlipFlop { .. } => None,
                Conjunction { name, .. } => Some(*name),
                Plain { .. } => None,
            })
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

    fn push_button(&mut self) -> Presses {
        use Module::*;
        use Pulse::*;

        let mut queue = VecDeque::from_iter([("button", "broadcaster", Low)]);
        let mut presses = Presses::default();

        while let Some((from, to, pulse)) = queue.pop_front() {
            // eprintln!("{from} --{pulse:?}--> {to}");

            match pulse {
                Low => presses.low += 1,
                High => presses.high += 1,
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

        presses
    }
}

#[derive(Debug)]
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

    fn outputs(&self) -> &[&'a str] {
        use Module::*;

        match self {
            FlipFlop { outputs, .. } | Conjunction { outputs, .. } | Plain { outputs, .. } => {
                outputs
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Pulse {
    Low,
    High,
}

#[derive(Debug, Copy, Clone, Default)]
struct Presses {
    low: usize,
    high: usize,
}

impl Presses {
    fn product(&self) -> usize {
        let Self { low, high } = self;
        low * high
    }
}

impl iter::Sum for Presses {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut this = Self::default();
        for p in iter {
            this.low += p.low;
            this.high += p.high;
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
