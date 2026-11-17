/// A `Band::max_dist` that covers the whole board, however large.
pub const ANYWHERE: u32 = u32::MAX;

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
    /// Fights no one but heals touching allies. Moves like `Basic` for now,
    /// until `FollowAlly` and `Retreat` are implemented.
    Healer,
    /// Deals no damage. Each round, converts the weakest touching enemy to its team.
    Priest,
    /// Tough fighter. Each touching ally deals +1 damage while it stands next to it.
    Commander,
}

/// Bands shared by every kind until kind-specific behavior exists.
const CHASE: &[Band] = &[
    Band {
        max_dist: ANYWHERE,
        action: Action::Advance,
    },
    Band {
        max_dist: ANYWHERE,
        action: Action::Sidestep,
    },
];

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
    /// Extra strength each touching ally deals this round (only allies that
    /// already have strength gain it).
    pub boost: u32,
    /// Whether this unit flips its weakest touching enemy to its team each round.
    pub converts: bool,
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
    /// Every kind, in the order the placement UI cycles through them.
    pub const ALL: [UnitKind; 4] = [
        UnitKind::Basic,
        UnitKind::Healer,
        UnitKind::Priest,
        UnitKind::Commander,
    ];

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            UnitKind::Basic => "Basic",
            UnitKind::Healer => "Healer",
            UnitKind::Priest => "Priest",
            UnitKind::Commander => "Commander",
        }
    }

    /// todo
    pub fn stats(self) -> Stats {
        match self {
            UnitKind::Basic => Stats {
                max_health: 4,
                cost: 1,
                strength: 1,
                heal: 0,
                boost: 0,
                converts: false,
                bands: CHASE,
            },
            UnitKind::Healer => Stats {
                max_health: 3,
                cost: 2,
                strength: 0,
                heal: 1,
                boost: 0,
                converts: false,
                bands: CHASE,
            },
            UnitKind::Priest => Stats {
                max_health: 3,
                cost: 3,
                strength: 0,
                heal: 0,
                boost: 0,
                converts: true,
                bands: CHASE,
            },
            UnitKind::Commander => Stats {
                max_health: 8,
                cost: 5,
                strength: 1,
                heal: 0,
                boost: 1,
                converts: false,
                bands: CHASE,
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

    #[test]
    fn healer_heals_and_does_not_fight() {
        let stats = UnitKind::Healer.stats();
        assert_eq!(stats.strength, 0);
        assert!(stats.heal > 0);
    }

    #[test]
    fn all_lists_every_kind_once() {
        assert!(UnitKind::ALL.contains(&UnitKind::Basic));
        assert!(UnitKind::ALL.contains(&UnitKind::Healer));
        assert!(UnitKind::ALL.contains(&UnitKind::Priest));
        assert!(UnitKind::ALL.contains(&UnitKind::Commander));
        assert_eq!(UnitKind::ALL.len(), 4);
    }

    #[test]
    fn priest_converts_and_does_not_fight() {
        let stats = UnitKind::Priest.stats();
        assert!(stats.converts);
        assert_eq!(stats.strength, 0);
    }

    #[test]
    fn commander_is_tough_and_boosts() {
        let stats = UnitKind::Commander.stats();
        assert!(stats.max_health > UnitKind::Basic.stats().max_health);
        assert!(stats.boost > 0);
    }
}
