/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// todo
    Advance,
    /// todo
    Retreat,
    /// todo
    Sidestep,
    /// todo
    FollowAlly,
    /// todo
    Hold,
}

/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    /// todo
    Red,
    /// todo
    Blue,
}

/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    /// todo
    Basic,
}

/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band {
    /// todo
    pub max_dist: u32,
    /// todo
    pub action: Action,
}

/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    /// todo
    pub max_health: u32,
    /// todo
    pub cost: u32,
    /// todo
    pub strength: u32,
    /// todo
    pub heal: u32,
    /// todo
    pub bands: &'static [Band],
}

/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    /// todo
    pub team: Team,
    /// todo
    pub kind: UnitKind,
    /// todo
    pub health: u32,
}

impl UnitKind {
    /// todo
    pub fn stats(self) -> Stats {
        match self {
            UnitKind::Basic => Stats {
                max_health: 4,
                cost: 1,
                strength: 1,
                heal: 0,
                bands: &[
                    Band {
                        max_dist: 8,
                        action: Action::Advance,
                    },
                    Band {
                        max_dist: 8,
                        action: Action::Sidestep,
                    },
                ],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_costs_one() {
        assert_eq!(UnitKind::Basic.stats().cost, 1);
    }
}
