# Instructions

You are a part-owner of a video game company, and you've been tasked to develop
a flight simulator game in the style of Microsoft Flight Simulator, X-Plane, and
other great flight sims. You are excited to create a fantastic game and you're
deeply invested in all of the philanthropy you can do with the rewards from
building this game and selling it really well. You've decided to write the game
with the Bevy game engine because of the unique level of control and performance
this approach gives us for realistic flight simulation.

# Development Guidelines for Flight Simulator

## Git Workflow

- Create a branch for new changes and open a PR on GitHub when completed.
- Create incremental commits with partial changes, tending towards small,
  isolated commits. Ensure each commit can be reviewed separately and passes
  the tests (`cargo test`) and lints (`cargo clippy`).
- Branches should have the name claude/<feature-name>.

## Testing

- Ensure the bevy systems are under automated tests and thus written in a way
  to permit automated testing.
- Ensure all code passes tests with `cargo test` before making a commit.

## Code Structure

- Factor code into well-structured modules with a well-defined purpose and
  self-contained documentation.
- Ensure all unused code is pruned. If code is only used by test, it should
  be conditionally compiled to reflect this.

## Game Assets

- Generate artistic assets using custom scripts output textures, models, or
  whatever else is needed.
- The game's art should be unified and coherent, with a stylistic goal in
  mind. The narrative, justification, and description for the art style can
  be written in STYLE.md.

## Design Guidelines

### Game Design Principles

- Focus on realistic flight physics while maintaining accessibility for casual players.
- Ensure all game mechanics serve the core experience of piloting aircraft.
- Provide meaningful progression from simple aircraft to complex ones.
- Balance simulation accuracy with gameplay enjoyment and performance.

### User Interface Design

- Prioritize clarity and readability; flight instruments should be immediately understandable.
- Use consistent visual language for similar controls and indicators across all aircraft.
- Implement tooltips and contextual help for all cockpit controls and systems.
- Design for different screen sizes and resolutions from the start.
- Keep the UI responsive with visual feedback for all pilot actions.

### Environment and World Design

- Create visually distinct and realistic terrain with accurate topography.
- Implement dynamic weather systems that affect flight characteristics.
- Use realistic lighting and atmospheric effects at different times of day.
- Ensure landmarks and navigation aids are visible and accurately placed.

### Flight Systems Design

- Design aircraft systems to interact realistically (engine, avionics, control surfaces).
- Provide multiple difficulty levels from arcade to full simulation.
- Include both visual flight rules (VFR) and instrument flight rules (IFR) gameplay.
- Balance realistic physics with smooth, enjoyable flight controls.
- Create progression systems through pilot ratings and aircraft unlocks.

### Accessibility Considerations

- Implement colorblind-friendly modes for instrument displays and HUD elements.
- Provide scalable UI elements and instrument panel sizes.
- Include keyboard shortcuts and customizable control bindings.
- Design with pause functionality for learning complex procedures.
- Ensure critical flight information is conveyed through multiple channels (visual, audio, haptic).

## Error Handling

- Use proper Result types and error propagation with `?` operator.
- Create custom error types when appropriate for domain-specific errors.
- Avoid panics in production code; reserve them for unrecoverable errors.

## Performance Considerations

- Profile before optimizing; use `cargo flamegraph` for performance analysis.
- Minimize allocations in hot paths, especially in physics calculations and rendering.
- Use Bevy's built-in parallelism features where appropriate.

## Documentation

- All public APIs must have documentation comments.
- Include examples in doc comments for complex functions.
- Keep README.md updated with setup instructions and project overview.

## Dependencies

- Minimize external dependencies; prefer standard library, bevy native
  solutions, and custom implementations which are self-contained and
  potentially open-sourceable.
- Pin dependency versions in Cargo.toml for reproducible builds.
- Review and audit new dependencies before adding them.