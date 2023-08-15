use crate::game::player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Upgrade
{
    Sight,
    Rifle,
    Drill,
    Wheel,

    Antenna,
    Battery,

    Heal,
}

impl Upgrade
{
    #[allow(dead_code)]
    pub(crate) fn cost(
        &self,
        player::stats::Stats {
            drill_level,
            rifle_level,
            wheel_level,
            sight_level,
            ..
        }: &player::stats::Stats,
    ) -> Option<Cost>
    {
        use crate::constants::upgrade;

        match self {
            Upgrade::Sight => upgrade::SIGHT_COSTS.get(*sight_level as usize).copied(),
            Upgrade::Rifle => upgrade::RIFLE_COSTS.get(*rifle_level as usize).copied(),
            Upgrade::Drill => upgrade::DRILL_COSTS.get(*drill_level as usize).copied(),
            Upgrade::Wheel => upgrade::WHEEL_COSTS.get(*wheel_level as usize).copied(),

            Upgrade::Antenna => Some(upgrade::ANTENNA_COST),
            Upgrade::Battery => Some(upgrade::BATTERY_COST),

            Upgrade::Heal => Some(upgrade::HEAL_COST),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Cost
{
    pub(crate) iron: u16,
    pub(crate) osmium: u16,
}
