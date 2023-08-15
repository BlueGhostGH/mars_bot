use crate::{
    constants,
    game::{self, upgrade},
};

pub(crate) use inventory::Inventory;
pub(crate) use stats::Stats;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Player
{
    pub(crate) position: game::Position,
    pub(crate) stats: Stats,
    pub(crate) inventory: Inventory,

    pub(crate) base: game::Position,
}

impl Player
{
    pub(crate) fn can_upgrade(&self) -> bool
    {
        self.position == self.base || self.stats.has_battery
    }

    pub(crate) fn can_afford(&self, upgrade: game::Upgrade) -> bool
    {
        use game::Upgrade as U;

        let threshold = match upgrade {
            U::Sight => constants::upgrade::SIGHT_THRESHOLDS
                .get(self.stats.sight_level as usize)
                .copied(),
            U::Rifle => constants::upgrade::RIFLE_THRESHOLDS
                .get(self.stats.rifle_level as usize)
                .copied(),
            U::Drill => constants::upgrade::DRILL_THRESHOLDS
                .get(self.stats.drill_level as usize)
                .copied(),
            U::Wheel => constants::upgrade::WHEEL_THRESHOLDS
                .get(self.stats.wheel_level as usize)
                .copied(),

            U::Antenna => Some(constants::upgrade::ANTENNA_THRESHOLD),
            U::Battery => Some(constants::upgrade::BATTERY_THRESHOLD),

            U::Heal => Some(constants::upgrade::HEAL_THRESHOLD),
        };

        threshold.is_some_and(|upgrade::Cost { iron, osmium }| {
            self.inventory.iron >= iron && self.inventory.osmium >= osmium
        })
    }
}

pub(crate) mod stats
{
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(crate) struct Stats
    {
        pub(crate) hit_points: u8,

        pub(crate) drill_level: u8,
        pub(crate) rifle_level: u8,
        pub(crate) wheel_level: u8,
        pub(crate) sight_level: u8,

        pub(crate) has_antenna: bool,
        pub(crate) has_battery: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Property
    {
        HitPoints,

        DrillLevel,
        RifleLevel,
        WheelLevel,
        SightLevel,

        HasAntenna,
        HasBattery,
    }

    pub(crate) const PROPERTIES: [Property; 7] = [
        Property::HitPoints,
        Property::DrillLevel,
        Property::RifleLevel,
        Property::WheelLevel,
        Property::SightLevel,
        Property::HasAntenna,
        Property::HasBattery,
    ];

    impl ::core::fmt::Display for Property
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
        {
            match self {
                Property::HitPoints => write!(f, "hit points"),

                Property::DrillLevel => write!(f, "drill level"),
                Property::RifleLevel => write!(f, "rifle level"),
                Property::WheelLevel => write!(f, "wheel level"),
                Property::SightLevel => write!(f, "sight level"),

                Property::HasAntenna => write!(f, "has antenna"),
                Property::HasBattery => write!(f, "has battery"),
            }
        }
    }
}

pub(crate) mod inventory
{
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(crate) struct Inventory
    {
        pub(crate) stone: u16,
        pub(crate) iron: u16,
        pub(crate) osmium: u16,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Item
    {
        Stone,
        Iron,
        Osmium,
    }

    pub(crate) const ITEMS: [Item; 3] = [Item::Stone, Item::Iron, Item::Osmium];

    impl ::core::fmt::Display for Item
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
        {
            match self {
                Item::Stone => write!(f, "stone"),
                Item::Iron => write!(f, "iron"),
                Item::Osmium => write!(f, "osmium"),
            }
        }
    }
}
