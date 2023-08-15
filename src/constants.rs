#![allow(dead_code)]

pub(crate) mod tile_weights
{
    pub(crate) const AIR: usize = 5;
    pub(crate) const BASE: usize = 5;

    pub(crate) const OSMIUM: usize = 2;
    pub(crate) const IRON: usize = 4;
    pub(crate) const STONE: usize = 8;
    pub(crate) const COBBLESTONE: usize = 8;

    pub(crate) const ACID: usize = 100;

    pub(crate) const PLAYER: usize = 30;

    pub(crate) const FOG: usize = 6;
}

pub(crate) mod upgrade
{
    use crate::game::upgrade;

    pub(crate) const QUEUE: [upgrade::Upgrade; 10] = const {
        use upgrade::Upgrade as U;

        [
            U::Battery,
            U::Sight,
            U::Wheel,
            U::Rifle,
            U::Sight,
            U::Rifle,
            U::Wheel,
            U::Antenna,
            U::Drill,
            U::Drill,
        ]
    };

    const fn cost(iron: u16, osmium: u16) -> upgrade::Cost
    {
        upgrade::Cost { iron, osmium }
    }

    pub(crate) const SIGHT_COSTS: [upgrade::Cost; 3] = [cost(0, 0), cost(3, 0), cost(6, 1)];
    pub(crate) const SIGHT_THRESHOLDS: [upgrade::Cost; 3] = [cost(0, 0), cost(3, 0), cost(6, 1)];

    pub(crate) const RIFLE_COSTS: [upgrade::Cost; 3] = [cost(0, 0), cost(3, 0), cost(6, 1)];
    pub(crate) const RIFLE_THRESHOLDS: [upgrade::Cost; 3] = [cost(0, 0), cost(3, 0), cost(6, 1)];

    pub(crate) const DRILL_COSTS: [upgrade::Cost; 3] = [cost(0, 0), cost(3, 0), cost(6, 1)];
    pub(crate) const DRILL_THRESHOLDS: [upgrade::Cost; 3] =
        [cost(0, 0), cost(3, 100), cost(6, 100)];

    pub(crate) const WHEEL_COSTS: [upgrade::Cost; 3] = [cost(0, 0), cost(3, 0), cost(6, 1)];
    pub(crate) const WHEEL_THRESHOLDS: [upgrade::Cost; 3] = [cost(0, 0), cost(3, 0), cost(6, 3)];

    pub(crate) const ANTENNA_COST: upgrade::Cost = cost(2, 1);
    pub(crate) const ANTENNA_THRESHOLD: upgrade::Cost = cost(2, 1);
    pub(crate) const BATTERY_COST: upgrade::Cost = cost(1, 1);
    pub(crate) const BATTERY_THRESHOLD: upgrade::Cost = cost(1, 1);

    pub(crate) const HEAL_COST: upgrade::Cost = cost(0, 1);
    pub(crate) const HEAL_THRESHOLD: upgrade::Cost = cost(0, 1);
}

pub(crate) mod acid
{
    pub(crate) const START_TURN: usize = 150;
    pub(crate) const TICK_RATE: usize = 2;
}
