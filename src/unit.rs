/// A `Band::max_dist` that covers the whole board, however large.
pub const ANYWHERE: u32 = u32::MAX;

/// What a unit does when one of its bands applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Step to a free neighboring cell strictly closer to the nearest enemy.
    Advance,
    /// Step away from the nearest enemy. Not implemented yet.
    Retreat,
    /// Step to a free neighboring cell at the same distance from the nearest enemy.
    Sidestep,
    /// Step toward the nearest ally. Not implemented yet.
    FollowAlly,
    /// Stay put. Ends the band walk without moving.
    Hold,
}

/// One of the two sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    /// Starts on the right half of the board.
    Red,
    /// Starts on the left half of the board.
    Blue,
}

impl Team {
    /// The other side.
    pub fn opponent(self) -> Team {
        match self {
            Team::Red => Team::Blue,
            Team::Blue => Team::Red,
        }
    }
}

/// The type of a unit, which decides its stats and behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    /// Plain fighter: damages touching enemies.
    Basic,
    /// Fights no one but heals touching allies. Moves like `Basic` for now,
    /// until `FollowAlly` and `Retreat` are implemented.
    Healer,
    /// Deals no damage. Each round, converts every touching enemy to its team.
    Priest,
    /// Tough fighter. Each touching ally deals +1 damage and heals +1 more (if it
    /// already damages or heals) while it stands next to it.
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

/// A distance range and the action taken inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band {
    /// Applies when the nearest enemy is at most this far away (Chebyshev).
    pub max_dist: u32,
    /// What the unit does when this band applies.
    pub action: Action,
}

/// Fixed numbers that define a unit kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    /// Health a unit starts with and can never be healed above.
    pub max_health: u32,
    /// Budget points needed to place one.
    pub cost: u32,
    /// Damage dealt each round to every touching enemy.
    pub strength: u32,
    /// Health restored each round to every touching ally.
    pub heal: u32,
    /// Extra damage and healing each touching ally gives this round (only allies
    /// that already deal damage or heal gain it).
    pub boost: u32,
    /// Whether this unit flips every touching enemy to its team each round.
    pub converts: bool,
    /// Movement rules in priority order. The first band that applies and can act wins.
    pub bands: &'static [Band],
}

/// A unit on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    /// The side it fights for.
    pub team: Team,
    /// Its type.
    pub kind: UnitKind,
    /// Current health. The unit is removed at 0.
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

    /// The fixed stats for this kind.
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
                max_health: 5,
                cost: 4,
                strength: 0,
                heal: 0,
                boost: 0,
                converts: true,
                bands: CHASE,
            },
            UnitKind::Commander => Stats {
                max_health: 6,
                cost: 6,
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
