use snafu::prelude::*;
use std::{collections::BTreeMap, ops, str::FromStr};

const INPUT: &str = include_str!("../input");

fn main() -> Result<(), Error> {
    let sum = sum_of_accepted_part_ratings(INPUT)?;
    // Part 1: 487623
    println!("{sum}");

    let combos = combinations_accepted_parts(INPUT)?;
    // Part 2: 113550238315130
    println!("{combos}");

    Ok(())
}

fn sum_of_accepted_part_ratings(s: &str) -> Result<u64, Error> {
    let (workflows, parts) = parse_input(s)?;

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

fn combinations_accepted_parts(s: &str) -> Result<u64, Error> {
    let (workflows, _parts) = parse_input(s)?;

    let mut queue = vec![("in", Restrictions::new())];
    let mut accepts = vec![];

    while let Some((node, group)) = queue.pop() {
        let wf = &workflows[node];

        let mut group = group;

        for rule in &wf.rules {
            let Rule {
                category,
                operator,
                value,
                action,
            } = *rule;
            let (passed, left) = group.split_at(category, operator, value);

            match action {
                Action::Accept => accepts.push(passed),
                Action::Reject => {}
                Action::Move(next_group) => queue.push((next_group, passed)),
            }

            group = left;
        }

        match wf.final_action {
            Action::Accept => accepts.push(group),
            Action::Reject => {}
            Action::Move(next_group) => queue.push((next_group, group)),
        }
    }

    Ok(accepts.iter().map(|r| r.count()).sum::<u64>())
}

fn parse_input(s: &str) -> Result<(BTreeMap<&str, Workflow<'_>>, Vec<Part>), Error> {
    let (workflows, parts) = s.split_once("\n\n").context(MalformedSnafu)?;

    let workflows = workflows
        .trim()
        .lines()
        .map(|workflow| Workflow::try_from(workflow).context(WorkflowSnafu { workflow }))
        .map(|wf| wf.map(|wf| (wf.name, wf)))
        .collect::<Result<_, _>>()?;

    let parts = parts
        .trim()
        .lines()
        .map(|part| Part::from_str(part).context(PartSnafu { part }))
        .collect::<Result<_, _>>()?;

    Ok((workflows, parts))
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
        use Operator::*;

        let Self {
            category,
            operator,
            value,
            ..
        } = *self;

        let part_value = part.0[category];

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
struct Part(XMAS<u64>);

impl Part {
    fn total_rating(&self) -> u64 {
        let XMAS { x, m, a, s } = self.0;
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

        Ok(Part(XMAS { x, m, a, s }))
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

/// Stores the ranges as (gte, lte)
#[derive(Debug, Copy, Clone, PartialEq)]
struct Restrictions(XMAS<(u64, u64)>);

impl Restrictions {
    fn new() -> Self {
        Self(XMAS::new((1, 4000)))
    }

    fn split_at(self, category: Category, operator: Operator, value: u64) -> (Self, Self) {
        use Operator::*;

        let mut kept = self;
        let mut rest = self;

        let kept_range = &mut kept.0[category];
        let rest_range = &mut rest.0[category];

        match operator {
            LessThan => {
                kept_range.1 = value - 1; // We track inclusive ranges
                rest_range.0 = value;
            }
            GreaterThan => {
                kept_range.0 = value + 1; // We track inclusive ranges
                rest_range.1 = value;
            }
        }

        (kept, rest)
    }

    fn count(&self) -> u64 {
        let XMAS { x, m, a, s } = self
            .0
            .map(|(s, e)| e.checked_sub(s).map(|v| v + 1).unwrap_or(0)); // + 1 for inclusive

        x * m * a * s
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
struct XMAS<T> {
    x: T,
    m: T,
    a: T,
    s: T,
}

impl<T> XMAS<T>
where
    T: Clone,
{
    fn new(v: T) -> Self {
        Self {
            x: v.clone(),
            m: v.clone(),
            a: v.clone(),
            s: v,
        }
    }
}

impl<T> XMAS<T> {
    fn map<F, U>(self, f: F) -> XMAS<U>
    where
        F: FnMut(T) -> U,
    {
        let Self { x, m, a, s } = self;
        let [x, m, a, s] = [x, m, a, s].map(f);
        XMAS { x, m, a, s }
    }
}

impl<T> ops::Index<Category> for XMAS<T> {
    type Output = T;

    fn index(&self, index: Category) -> &Self::Output {
        use category_shorthand::*;
        match index {
            X => &self.x,
            M => &self.m,
            A => &self.a,
            S => &self.s,
        }
    }
}

impl<T> ops::IndexMut<Category> for XMAS<T> {
    fn index_mut(&mut self, index: Category) -> &mut Self::Output {
        use category_shorthand::*;
        match index {
            X => &mut self.x,
            M => &mut self.m,
            A => &mut self.a,
            S => &mut self.s,
        }
    }
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

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(
            167409079868000,
            combinations_accepted_parts(EXAMPLE_INPUT_1)?
        );

        Ok(())
    }

    #[test]
    fn restrictions_split_at() {
        use {category_shorthand::*, Operator::*};

        let r = Restrictions::new();
        let (a, b) = r.split_at(X, LessThan, 2000);
        assert_eq!((1, 1999), a.0.x);
        assert_eq!((2000, 4000), b.0.x);

        let r = Restrictions::new();
        let (a, b) = r.split_at(X, GreaterThan, 2000);
        assert_eq!((2001, 4000), a.0.x);
        assert_eq!((1, 2000), b.0.x);
    }

    #[test]
    fn restrictions_count() {
        use {category_shorthand::*, Operator::*};

        const ALL_POSSIBILITIES: u64 = 4000 * 4000 * 4000 * 4000;

        let r = Restrictions::new();
        assert_eq!(ALL_POSSIBILITIES, r.count());

        // We aren't losing some when splitting
        for i in 2..=3999 {
            for op in [LessThan, GreaterThan] {
                let (a, b) = Restrictions::new().split_at(X, op, i);
                assert_eq!(
                    ALL_POSSIBILITIES,
                    a.count() + b.count(),
                    "i = {i}; op = {op:?}",
                );
            }
        }

        // Repeatedly split a few times
        let r = Restrictions::new();
        let (a, b) = r.split_at(X, LessThan, 1700);
        let (b, c) = b.split_at(M, LessThan, 1200);
        let (c, d) = c.split_at(A, GreaterThan, 900);
        let (d, e) = d.split_at(S, GreaterThan, 100);

        let all = [a, b, c, d, e];
        let sum = all.map(|r| r.count()).iter().sum();
        assert_eq!(ALL_POSSIBILITIES, sum);
    }
}
