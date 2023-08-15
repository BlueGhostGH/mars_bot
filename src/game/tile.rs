#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) enum Tile
{
    Air,
    Base,

    Cobblestone,
    Stone,
    Iron,
    Osmium,

    Bedrock,
    Acid,

    Player
    {
        id: u8,
    },

    #[default]
    Fog,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) enum NonPlayerTile
{
    #[allow(dead_code)]
    Air,
    #[allow(dead_code)]
    Base,

    Cobblestone,
    Stone,
    Iron,
    Osmium,

    #[allow(dead_code)]
    Bedrock,
    #[allow(dead_code)]
    Acid,

    #[default]
    Fog,
}

impl NonPlayerTile
{
    pub(crate) fn is_obstacle(&self) -> bool
    {
        match self {
            NonPlayerTile::Acid
            | NonPlayerTile::Bedrock
            | NonPlayerTile::Cobblestone
            | NonPlayerTile::Stone => true,
            _ => false,
        }
    }
}

impl From<NonPlayerTile> for Tile
{
    fn from(non_player_tile: NonPlayerTile) -> Self
    {
        match non_player_tile {
            NonPlayerTile::Air => Tile::Air,
            NonPlayerTile::Stone => Tile::Stone,
            NonPlayerTile::Cobblestone => Tile::Cobblestone,
            NonPlayerTile::Bedrock => Tile::Bedrock,
            NonPlayerTile::Iron => Tile::Iron,
            NonPlayerTile::Osmium => Tile::Osmium,
            NonPlayerTile::Base => Tile::Base,
            NonPlayerTile::Acid => Tile::Acid,

            NonPlayerTile::Fog => Tile::Fog,
        }
    }
}

impl TryFrom<Tile> for NonPlayerTile
{
    type Error = Tile;

    fn try_from(tile: Tile) -> Result<Self, Self::Error>
    {
        match tile {
            Tile::Air => Ok(NonPlayerTile::Air),
            Tile::Stone => Ok(NonPlayerTile::Stone),
            Tile::Cobblestone => Ok(NonPlayerTile::Cobblestone),
            Tile::Bedrock => Ok(NonPlayerTile::Bedrock),
            Tile::Iron => Ok(NonPlayerTile::Iron),
            Tile::Osmium => Ok(NonPlayerTile::Osmium),
            Tile::Base => Ok(NonPlayerTile::Base),
            Tile::Acid => Ok(NonPlayerTile::Acid),

            _ => Err(tile),
        }
    }
}

impl PartialEq<NonPlayerTile> for Tile
{
    fn eq(&self, np_tile: &NonPlayerTile) -> bool
    {
        use NonPlayerTile as NPTile;

        matches!(
            (self, np_tile),
            (Tile::Air, NPTile::Air)
                | (Tile::Base, NPTile::Base)
                | (Tile::Cobblestone, NPTile::Cobblestone)
                | (Tile::Stone, NPTile::Stone)
                | (Tile::Iron, NPTile::Iron)
                | (Tile::Osmium, NPTile::Osmium)
                | (Tile::Bedrock, NPTile::Bedrock)
                | (Tile::Acid, NPTile::Acid)
                | (Tile::Fog, NPTile::Fog)
        )
    }
}

pub(crate) trait Pattern
{
    fn matches(&self, np_tile: &NonPlayerTile) -> bool;
}

impl Pattern for NonPlayerTile
{
    fn matches(&self, np_tile: &NonPlayerTile) -> bool
    {
        self == np_tile
    }
}

impl<F> Pattern for F
where
    F: Fn(&NonPlayerTile) -> bool,
{
    fn matches(&self, np_tile: &NonPlayerTile) -> bool
    {
        self(np_tile)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) enum ViableTile
{
    Air,
    Base,

    Cobblestone,
    Stone,
    Iron,
    Osmium,

    Acid,

    Player,

    #[default]
    Fog,
}

impl ViableTile
{
    pub(crate) fn requires_mining(&self) -> bool
    {
        match self {
            ViableTile::Stone | ViableTile::Cobblestone | ViableTile::Iron | ViableTile::Osmium => {
                true
            }
            ViableTile::Fog
            | ViableTile::Air
            | ViableTile::Base
            | ViableTile::Acid
            | ViableTile::Player => false,
        }
    }

    pub(crate) fn weight(&self) -> usize
    {
        use crate::constants::tile_weights;

        match self {
            ViableTile::Air => tile_weights::AIR,
            ViableTile::Base => tile_weights::BASE,

            ViableTile::Osmium => tile_weights::OSMIUM,
            ViableTile::Iron => tile_weights::IRON,
            ViableTile::Stone => tile_weights::STONE,
            ViableTile::Cobblestone => tile_weights::COBBLESTONE,

            ViableTile::Acid => tile_weights::ACID,

            ViableTile::Player => tile_weights::PLAYER,

            ViableTile::Fog => tile_weights::FOG,
        }
    }
}

impl TryFrom<Tile> for ViableTile
{
    type Error = Option<::core::convert::Infallible>;

    fn try_from(tile: Tile) -> Result<Self, Self::Error>
    {
        match tile {
            Tile::Air => Ok(ViableTile::Air),
            Tile::Base => Ok(ViableTile::Base),

            Tile::Cobblestone => Ok(ViableTile::Cobblestone),
            Tile::Stone => Ok(ViableTile::Stone),
            Tile::Iron => Ok(ViableTile::Iron),
            Tile::Osmium => Ok(ViableTile::Osmium),

            Tile::Acid => Ok(ViableTile::Acid),

            Tile::Player { .. } => Ok(ViableTile::Player),

            Tile::Fog => Ok(ViableTile::Fog),

            _ => Err(None),
        }
    }
}
