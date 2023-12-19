use snafu::prelude::*;
use std::{collections::BTreeMap, str::FromStr};

const INPUT: &str = include_str!("../input");

fn main() -> Result<(), Error> {
    let sum = sum_of_accepted_part_ratings(INPUT)?;
    // Part 1: 487623
    println!("{sum}");

    Ok(())
}

fn sum_of_accepted_part_ratings(s: &str) -> Result<u64, Error> {
    let (workflows, parts) = s.split_once("\n\n").context(MalformedSnafu)?;

    let workflows = workflows
        .trim()
        .lines()
        .map(|workflow| Workflow::try_from(workflow).context(WorkflowSnafu { workflow }))
        .map(|wf| wf.map(|wf| (wf.name, wf)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    let parts = parts
        .trim()
        .lines()
        .map(|part| Part::from_str(part).context(PartSnafu { part }))
        .collect::<Result<Vec<_>, _>>()?;

    let accepted = parts.into_iter().filter(|part| {
        let mut name = "in";

        loop {
            match workflows[name].action_for(part) {
                Action::Accept => break true,
                Action::Reject => break false,
                Action::Move(next_name) => name = next_name,
            }
        }
    });

    let total_rating = accepted.map(|part| part.total_rating()).sum();

    Ok(total_rating)
}

#[derive(Debug, Snafu)]
enum Error {
    Malformed,

    Workflow {
        source: ParseWorkflowError,
        workflow: String,
    },

    Part {
        source: ParsePartError,
        part: String,
    },
}

#[derive(Debug)]
struct Workflow<'a> {
    name: &'a str,
    rules: Vec<Rule<'a>>,
    final_action: Action<'a>,
}

impl Workflow<'_> {
    fn action_for(&self, part: &Part) -> &Action<'_> {
        self.rules
            .iter()
            .find(|rule| rule.passes(part))
            .map(|rule| &rule.action)
            .unwrap_or(&self.final_action)
    }
}

impl<'a> TryFrom<&'a str> for Workflow<'a> {
    type Error = ParseWorkflowError;

    // px{a<2006:qkq,m>2090:A,rfg}
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        use parse_workflow_error::*;

        let (name, l) = value.split_once('{').context(MalformedSnafu)?;
        let rules = l.trim_end_matches('}');
        let mut rules = rules.split(',');
        let final_action = rules.next_back().context(FinalActionMissingSnafu)?;

        let rules = rules
            .map(|rule| Rule::try_from(rule).context(RuleSnafu { rule }))
            .collect::<Result<_, _>>()?;

        let final_action = Action::from(final_action);

        Ok(Self {
            name,
            rules,
            final_action,
        })
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseWorkflowError {
    Malformed,

    FinalActionMissing,

    Rule {
        source: ParseRuleError,
        rule: String,
    },
}

#[derive(Debug, Copy, Clone)]
struct Rule<'a> {
    category: Category,
    operator: Operator,
    value: u64,
    action: Action<'a>,
}

impl Rule<'_> {
    fn passes(&self, part: &Part) -> bool {
        use category_shorthand::*;
        use Operator::*;

        let Self {
            category,
            operator,
            value,
            ..
        } = *self;

        let part_value = match category {
            X => part.x,
            M => part.m,
            A => part.a,
            S => part.s,
        };

        match operator {
            LessThan => part_value < value,
            GreaterThan => part_value > value,
        }
    }
}

impl<'a> TryFrom<&'a str> for Rule<'a> {
    type Error = ParseRuleError;

    // a<2006:qkq
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        use parse_rule_error::*;

        let (condition, action) = value.split_once(':').context(MalformedSnafu)?;
        let (idx, operator) = condition
            .match_indices(&['<', '>'])
            .next()
            .context(Malformed2Snafu)?;

        let (category, t) = condition.split_at(idx);
        let value = &t[operator.len()..];

        let category = category.parse().context(CategorySnafu { category })?;
        let operator = operator.parse().context(OperatorSnafu { operator })?;
        let value = value.parse().context(ValueSnafu { value })?;
        let action = Action::from(action);

        Ok(Rule {
            category,
            operator,
            value,
            action,
        })
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseRuleError {
    Malformed,

    Malformed2,

    Category {
        source: ParseCategoryError,
        category: String,
    },

    Operator {
        source: ParseOperatorError,
        operator: String,
    },

    Value {
        source: std::num::ParseIntError,
        value: String,
    },
}

#[derive(Debug, Copy, Clone)]
enum Category {
    ExtremelyCoolLooking,
    Musical,
    Aerodynamic,
    Shiny,
}

impl FromStr for Category {
    type Err = ParseCategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "x" => Category::ExtremelyCoolLooking,
            "m" => Category::Musical,
            "a" => Category::Aerodynamic,
            "s" => Category::Shiny,
            _ => return ParseCategorySnafu.fail(),
        })
    }
}

#[derive(Debug, Snafu)]
struct ParseCategoryError;

mod category_shorthand {
    pub(super) use super::Category::{
        Aerodynamic as A, ExtremelyCoolLooking as X, Musical as M, Shiny as S,
    };
}

#[derive(Debug, Copy, Clone)]
enum Operator {
    LessThan,
    GreaterThan,
}

impl FromStr for Operator {
    type Err = ParseOperatorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "<" => Operator::LessThan,
            ">" => Operator::GreaterThan,
            _ => return ParseOperatorSnafu.fail(),
        })
    }
}

#[derive(Debug, Snafu)]
struct ParseOperatorError;

#[derive(Debug, Copy, Clone)]
enum Action<'a> {
    Accept,
    Reject,
    Move(&'a str),
}

impl<'a> From<&'a str> for Action<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "A" => Action::Accept,
            "R" => Action::Reject,
            _ => Action::Move(value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Part {
    x: u64,
    m: u64,
    a: u64,
    s: u64,
}

impl Part {
    fn total_rating(&self) -> u64 {
        let Self { x, m, a, s } = *self;
        x + m + a + s
    }
}

impl FromStr for Part {
    type Err = ParsePartError;

    // {x=787,m=2655,a=1222,s=2876}
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use parse_part_error::*;

        let l = s.trim_matches(&['{', '}']);
        let mut ratings = l
            .split(',')
            .map(|r| {
                use parse_rating_error::*;

                let (_, value) = r.split_once('=').context(MalformedSnafu)?;
                value.parse::<u64>().context(InvalidSnafu { value })
            })
            .map(|r| r.context(RatingSnafu));

        let x = ratings.next().context(MissingXSnafu)??;
        let m = ratings.next().context(MissingMSnafu)??;
        let a = ratings.next().context(MissingASnafu)??;
        let s = ratings.next().context(MissingSSnafu)??;

        Ok(Part { x, m, a, s })
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParsePartError {
    Rating { source: ParseRatingError },

    MissingX,
    MissingM,
    MissingA,
    MissingS,
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseRatingError {
    Malformed,
    Invalid {
        source: std::num::ParseIntError,
        value: String,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(19114, sum_of_accepted_part_ratings(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
