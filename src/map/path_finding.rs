use std::collections;

use crate::{
    io::output::moves,
    map::{self, direction, ParentData},
    position,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Path
{
    pub(crate) moves: moves::Moves,
    pub(crate) end_position: position::Position,
    pub(crate) mine_direction: Option<direction::Direction>,
}

impl map::Map
{
    pub(crate) fn find_path(&self, to: position::Position) -> Option<Path>
    {
        let mut location = to;
        let mut moves = collections::VecDeque::new();
        let mut mine_direction = None;

        while location != self.player.position {
            let entry = self.entry_at(location)?;

            let map::ParentData {
                direction_from_parent,
                parent_location,
                requires_mining,

                turn_move_index,
            } = entry.parent_data?;

            if let Some(map::ParentData {
                direction_from_parent,
                requires_mining,

                turn_move_index,
                ..
            }) = moves.front().copied()
            {
                if turn_move_index == 0 {
                    moves.clear();

                    mine_direction = if requires_mining {
                        Some(direction_from_parent)
                    } else {
                        None
                    };
                }
            }

            moves.push_front(map::ParentData {
                direction_from_parent,
                parent_location: location,
                requires_mining,

                turn_move_index,
            });

            location = parent_location;

            if moves.len() > self.player.wheel_level as _ {
                let _last = moves.pop_back();
            }
        }

        let end_position = moves.front().map_or(
            to,
            |&ParentData {
                 parent_location, ..
             }| parent_location,
        );

        let moves = moves::Moves {
            mvs: [moves.pop_front(), moves.pop_front(), moves.pop_front()].map(|mv| {
                if let Some(ParentData {
                    direction_from_parent,
                    ..
                }) = mv
                {
                    Some(direction_from_parent)
                } else {
                    None
                }
            }),
        };

        Some(Path {
            moves,
            end_position,
            mine_direction,
        })
    }
}