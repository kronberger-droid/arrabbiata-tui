# arrabbiata-tui

A terminal UI client for [Arrabbiata](https://github.com/wieseldinger/arrabbiata), a pomodoro timer with extra features.

Built with [Ratatui](https://ratatui.rs). Includes a Nix flake for NixOS integration.

## Configuration

The following environment variables must be set:

- `ARRABBIATA_API_URL` -- URL of the Arrabbiata API server
- `ARRABBIATA_USER_ID` -- your user ID
- `ARRABBIATA_FALLBACK_USER_ID` -- fallback user ID (optional if using Nix, defaults to `userId`)

When using the Nix flake, use `lib.withConfig` to bake these in:

```nix
arrabbiata-tui.lib.withConfig {
  system = "x86_64-linux";
  apiUrl = "http://your-server:5000/api/arrabbiata";
  userId = "your-user-id";
}
```
