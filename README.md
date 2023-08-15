# Mars Bot

### A small bot for [Mars Mission](https://github.com/meemknight/marsMission) written in Rust.

## Running the bot

The bot can be run with an optional path to the `marsMission` folder containing the `game` folder.

```bash
cargo run -- path/to/marsMission/out/build/x64-Debug
```

## TODO
 - [x] Path finding
   - [x] Compute weighted distances to all tiles
   - [x] Take wheel level into account
   - [x] Mine the tile ahead if in the way
 - [x] Resource gathering
 - [x] Upgrading
   - [x] Go through a queue of upgrades
     - [x] Get battery before everything else
   - [x] Take a heal upgrade if hit points are low
 - [x] Keep opponent cache
   - [x] Determine opponents wheel level
   - [ ] Determine opponents rifle level
 - [ ] Handle opponent encounters
   - [ ] Fight back if stronger or can't run away
   - [ ] Run away
     - [ ] Place block behind
 - [x] Acid handling
   - [x] Count turns
   - [x] Update internal map accordingly with acid level
   - [x] Run to the center of the map
 - [ ] Final phase (a.k.a caging)
   - [ ] Assemble cage
     - [ ] Block corners around entrance and clear centre
     - [ ] Move to centre, clear around and move to third spot
     - [ ] Block remaining corners and move back to center
     - [ ] Handle oncoming players
       - [ ] Handle players coming into empty spots
       - [ ] Handle players mining corners
 - [ ] Make use of knowledge that there is only one Osmium per chunk
 - [ ] Implement a graphical interface
   - [ ] Display bot state
   - [ ] Display bot logs
   - [ ] Display map known by the bot
     - [ ] Display opponents
   - [ ] Display path finding
     - [ ] Colour code weights
   - [ ] Allow changing constants/parameters(fine-tuning) of the bot
 - [ ] Implement profiles
 - [ ] Write unit tests
   - [ ] WIP...
 - [ ] Transition the `mars_bot` lib to `#![no_std]`

## License

This project is licensed under the [MIT No Attribution License](./LICENSE).