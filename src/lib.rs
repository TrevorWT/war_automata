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
pub struct Unit {
    /// todo
    pub team: Team,
    /// todo
    pub kind: UnitKind,
}

impl UnitKind {
    /// todo
    pub fn cost(self) -> u32 {
        match self {
            UnitKind::Basic => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn  basic_costs_one() {
        assert_eq!(UnitKind::Basic.cost(), 1);
    }
}
