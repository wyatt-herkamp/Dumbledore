# Dumbledore
[![Documentation](https://docs.rs/dumbledore/badge.svg)](https://docs.rs/dumbledore/)
[![Crates.io](https://img.shields.io/crates/v/dumbledore.svg)](https://crates.io/crates/dumbledore)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![Unsafe Rust](https://forthebadge.com/images/badges/powered-by-black-magic.svg)](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)

ECS for the game server


## Performance
 Dumbledore sacrifices some performance for allowing async calling. However, the performance seems to be comparable to other projects such as [hecs].


## Warning! Seriously read this!

This project is experimental and is designed to be used in a server environment.

Basically using async calls on a normal game client might not be optimal.

##### This is also my first attempt at unsafe Rust code.
If you have any suggestions, please let me know. I am open to any feedback on this project.


#### Other Projects I recommend:
- [hecs], Which I used as a reference for this project and is a great
  starting point for maintained ECS
- [bevy]
- [specs]
- [legion]


[bevy]: https://github.com/bevyengine/bevy

[specs]: https://github.com/amethyst/specs

[legion]: https://github.com/TomGillen/legion

[hecs]: https://github.com/Ralith/hecs


## Why is it named Dumbledore?

I used a random name generator also known as [peterhenryd](https://github.com/peterhenryd).

"it makes sense. an ecs is magical + controls everything" - peterhenryd


### Thanks!

Special thanks to [SanderMertens](https://github.com/SanderMertens) for his resources on ECS design.
