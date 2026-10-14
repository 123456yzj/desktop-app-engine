use crate::{
    description::{Description, Property, Step},
    validation,
};
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug)]
struct Invocation {
    action_id: String,
    snapshot: BTreeMap<String, String>,
}

pub struct Engine {
    description: Description,
    state: BTreeMap<String, String>,
    pending: VecDeque<Invocation>,
    error: Option<String>,
}

impl Engine {
    pub fn new(description: Description) -> Result<Self, String> {
        validation::validate(&description)?;
        Ok(Self {
            state: description.state.clone(),
            description,
            pending: VecDeque::new(),
            error: None,
        })
    }

    pub fn description(&self) -> &Description {
        &self.description
    }
    pub fn state(&self) -> &BTreeMap<String, String> {
        &self.state
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn resolve(&self, property: &Property) -> Result<String, String> {
        resolve(property, &self.state)
    }

    pub fn set_state(&mut self, key: &str, value: String) -> Result<(), String> {
        validation::string_size(&value, &format!("state[{key:?}]"))?;
        let slot = self
            .state
            .get_mut(key)
            .ok_or_else(|| format!("未定义状态 {key:?}"))?;
        *slot = value;
        Ok(())
    }

    /// Capture inputs now, not when the next render or queue processing occurs.
    pub fn trigger(&mut self, action_id: &str) -> Result<(), String> {
        if !self.description.actions.contains_key(action_id) {
            let error = format!("动作 {action_id:?}: 未定义动作");
            self.error = Some(error.clone());
            return Err(error);
        }
        self.pending.push_back(Invocation {
            action_id: action_id.into(),
            snapshot: self.state.clone(),
        });
        Ok(())
    }

    /// Drain events exactly once. Successful earlier writes remain on failure.
    pub fn process_pending(&mut self) {
        while let Some(invocation) = self.pending.pop_front() {
            self.error = None;
            let steps = self.description.actions[&invocation.action_id]
                .steps
                .clone();
            for (index, step) in steps.iter().enumerate() {
                if let Err(error) = self.execute(step, &invocation.snapshot) {
                    self.error = Some(format!(
                        "动作 {:?} steps[{index}]: {error}",
                        invocation.action_id
                    ));
                    break;
                }
            }
        }
    }

    fn execute(&mut self, step: &Step, snapshot: &BTreeMap<String, String>) -> Result<(), String> {
        let (target, value) = match step {
            Step::Set { target, value } => (target, resolve(value, snapshot)?),
            Step::Concat { target, parts } => {
                let mut value = String::new();
                for part in parts {
                    let next = resolve(part, snapshot)?;
                    if value.len() + next.len() > validation::MAX_STRING_BYTES {
                        return Err(format!(
                            "target {target:?}: 拼接结果超过 {} UTF-8 字节",
                            validation::MAX_STRING_BYTES
                        ));
                    }
                    value.push_str(&next);
                }
                (target, value)
            }
        };
        self.set_state(target, value)
    }
}

fn resolve(property: &Property, state: &BTreeMap<String, String>) -> Result<String, String> {
    match property {
        Property::Literal { value } => Ok(value.clone()),
        Property::State { key } => state
            .get(key)
            .cloned()
            .ok_or_else(|| format!("未定义状态 {key:?}")),
    }
}
