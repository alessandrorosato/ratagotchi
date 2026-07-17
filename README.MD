# Ratagotchi

![Project screenshot](./assets/screenshot-1.png)

Take care of a cute rat living in your terminal! Feed it, play with it and take care of it.

Written in _Rust_ 🦀 and powered by _Ratatui_ 🐀!

## How to Play

On first load, a new ratagotchi is spawned. From that moment, you need to take care of it so it can grow healthy. Vitals will decay over time, even when the game is not running.

## Features

Different ways to take care of your ratagotchi:

- **Feed**: offer either cheese (tastier but fattening) or premium ratfood (more filling, less rewarding)!
- **Play**: let the ratagotchi pick a number, then guess if the second one is going to be higher or lower than it.
- **Heal**: if your ratagotchi is feeling dizzy, it probably needs some care.
- **Bath**: ratagotchis tend to poop, keep the room tidy.
- **Scold**: it's very important to keep your ratagotchi well disciplined.

## Installation

Right now, you just have to clone the repo and build the project on your local. Binaries coming soon!



## ToDo

There's a bunch of stuff still missing, here's a definitely-not-exhaustive list of features I'd like to implement

### Gameplay

- [ ] Death conditions.
- [ ] "Win" conditions.
- [ ] Day/light cycle, sleep.
    - [ ] Light and ratagotchi reaction.
- [ ] Review all decay factors.
- [ ] Something related to low discipline.

### UI

- [ ] Redefine layouts to keep positioning of all elements consistent.
- [ ] Improve navigation button (maybe more square-y).
- [ ] Better bath "wave" (looks like a meatgrinder atm).
- [ ] Low health/disease cue.
- [ ] Play view:
    - [x] Lock choice once the second number is displayed.
    - [ ] At the end of the final round, remove the frame still in Play view.
- [ ] Visual ratagotchi variants based on its weight.


## Disclaimer

This project is mostly an excuse to learn and write some Rust, so it is very likely littered with unhortodox code and bad decisions. I encourage you to suggest changes in order to improve the code and expand my knowledge.