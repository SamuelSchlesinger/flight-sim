# Sky Hunter

A fast-paced arcade flight game built with the Bevy game engine, featuring aerial combat, power-ups, and upgrade progression.

## Features

- **Multiple Game Modes**
  - **Target Hunt**: Collect balloons for points while avoiding enemies
  - **Survival**: Fight endless waves of increasingly difficult enemies
  - **Time Attack**: Score as many points as possible before time runs out
  - **Free Flight**: Practice your flying skills without objectives

- **Dynamic Combat System**
  - Smart enemy AI with different behavior patterns
  - Multiple enemy types with unique tactics
  - Weapon systems including bullets and homing missiles
  - Power-ups for temporary advantages

- **Upgrade Progression**
  - Earn coins to upgrade your aircraft between runs
  - Improve speed, maneuverability, score multipliers, and magnet range
  - Persistent upgrades across game sessions

- **Power-Ups**
  - Health Pack: Restore health
  - Energy Recharge: Boost your speed
  - Rapid Fire: Increased fire rate
  - Shield: Temporary invincibility
  - Speed Boost: Enhanced movement speed
  - Triple Shot: Fire three bullets at once
  - Homing Missiles: Auto-targeting projectiles

## Screenshots

*Coming soon*

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/sky-hunter.git
cd sky-hunter
```

2. Build and run the game:
```bash
cargo run --release
```

### Controls

- **W/S** - Pitch up/down
- **A/D** - Roll left/right
- **Shift** - Speed boost
- **Space** - Fire weapons
- **ESC** - Pause/Menu
- **Mouse** - Camera control (right-click + drag)

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build (recommended for playing)
cargo build --release

# Run tests
cargo test

# Run linter
cargo clippy
```

### Project Structure

```
src/
├── main.rs           # Core game loop and aircraft controls
├── game_state.rs     # Game state management and progression
├── targets.rs        # Target spawning and collision detection
├── enemies.rs        # Enemy AI and combat systems
├── powerups.rs       # Power-up system implementation
└── ui.rs            # User interface and menus
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Bevy](https://bevyengine.org/) - A refreshingly simple data-driven game engine built in Rust
- UI powered by [egui](https://github.com/emilk/egui) - An immediate mode GUI library

## Roadmap

- [ ] Add sound effects and background music
- [ ] Implement more enemy types
- [ ] Create additional game modes
- [ ] Add multiplayer support
- [ ] Improve visual effects and particle systems
- [ ] Create more diverse environments
- [ ] Add gamepad support
- [ ] Implement replay system

## Known Issues

- Performance may vary on integrated graphics cards
- Some UI elements may not scale properly on very high resolution displays

## Support

If you encounter any issues or have suggestions, please file an issue on the [GitHub issue tracker](https://github.com/yourusername/sky-hunter/issues).