![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
[![Crates.io Version](https://img.shields.io/crates/v/firetail)](https://crates.io/crates/firetail)
[![ko-fi](https://shields.io/badge/ko--fi-Buy_me_a_coffee-ff5f5f?logo=ko-fi&style=for-the-badgeKo-fi)](https://ko-fi.com/vincenzomarturano02)
# About
`firetail` is a tui tool designed to filter and view your Opnsense firewall logs with ease, It's still in development, but it already offers key features for efficient log analysis.

Key features include:
- **Interactive viewing**: Use simple keybindings for efficient navigation.
- **Real-time log viewing**: Optionally view logs directly from the firewall.
  
With `firetail`, you can quickly parse and interact with firewall logs, ensuring you stay on top of your network security.

# Demo
![Alt Text](demo/demo.gif)
# Prerequisites
* A linux based OS.
* [Nerd Fonts](https://www.nerdfonts.com/) (optional, but recommended for proper icon rendering)

# Quickstart

You can install `firetail` with `cargo`:
```bash
cargo install firetail
```

# Usage
```
firetail [OPTIONS] [LOGFILE]
```
Use `--help` to print the help message.

> ## **Note**
> if LOGFILE is not provided **firetail** will get logs from **stdin**

# Examples
```bash
firetail -i vlan0.20 filter_20250102.log
```
Show logs filtered by the `vlan0.20` interface.

```bash
firetail -i vlan0.20,vlan0.10 --dst-ip 192.168.40.10 filter_20250102.log
```
Show logs filtered by the `vlan0.20` and `vlan0.10` interfaces, and the destination IP `192.168.40.10`.

> ## **TIP**
> ## Use this command to get logs directly from the firewall
> ```bash
>nohup ssh yourfirewall opnsense-log -f filter >/dev/null 2>&1 | firetail
>```
> 


# :keyboard: Keybindings

| Key                    | Action                                           |
|------------------------|--------------------------------------------------|
| `q`                    | Quit                                             |
| `Up` / `k`             | Scroll up                                        |
| `Down` / `j`           | Scroll down                                      |
| `End`                  | Scroll to end                                    |
| `.`                    | Enable auto-scroll                               |
| `i`                    | Toggle log info popup                            |
| `d`                    | Start date search (switches to edit mode)        |
| `Enter` (in edit mode) | Confirm edit and return to normal mode           |
| `Esc` (in edit mode)   | Cancel edit and return to normal mode            |

# Todos
- [ ] Allow to change filter settings in realtime

# Contributing
Contributions are very welcome and appreciated! Feel free to open an issue, submit a pull request, or suggest improvements. :rocket:

# Support
[![ko-fi](https://shields.io/badge/ko--fi-Buy_me_a_coffee-ff5f5f?logo=ko-fi&style=for-the-badgeKo-fi)](https://ko-fi.com/vincenzomarturano02)

If you like what I'm working on and want to support me, you can buy me a coffee! ☕ It helps me stay energized and motivated to keep improving `firetail`. Every cup is appreciated! 🙏

