# Curl Quests

[![Crates.io](https://img.shields.io/crates/v/curl-quests.svg)](https://crates.io/crates/curl-quests) [![Downloads](https://img.shields.io/crates/d/curl-quests.svg)](https://crates.io/crates/curl-quests)

<p align="center"><strong>⭐ If these quests helped you learn curl and HTTP, please consider starring the repo.</strong></p>

An interactive terminal game for learning `curl` and HTTP APIs through hands-on quests. Each quest spins up a real local server, gives you instructions, and verifies your work directly against the database no guessing, just doing.

---

### Requirements

- [Rust & Cargo](https://rustup.rs/): install via rustup (includes both)
- [curl](https://curl.se/download.html): usually pre-installed on macOS/Linux
- [jq](https://jqlang.github.io/jq/download/): required for Quest 11 onwards

Check your versions:

```sh
rustc --version
cargo --version
curl --version
jq --version
```

---

### Get started (two ways)

**Option A: Install via Cargo (recommended)**

```sh
cargo install curl-quests
curl-quests
```

**Option B: Clone and build manually**

```sh
git clone https://github.com/lite-quests/curl-quests.git
cd curl-quests
cargo build
cargo run
```

---

### Navigating the app

When you launch `curl-quests` you'll see a terminal UI with a quest grid and a top navigation bar.

| Key       | Action                                           |
| --------- | ------------------------------------------------ |
| `←` / `→` | Move between tabs (Levels / Instructions / Exit) |
| `Enter`   | Open selected tab or quest                       |
| `Esc`     | Go back / dismiss                                |
| `q`       | Quit                                             |

**Inside a quest:**

| Key                   | Action                                          |
| --------------------- | ----------------------------------------------- |
| `Tab` / `Shift+Tab`   | Switch focus between sections                   |
| `↑` / `↓`             | Scroll instructions or navigate command history |
| `Shift+↑` / `Shift+↓` | Scroll terminal output                          |
| `←` / `→`             | Resize the left/right columns                   |
| `Enter`               | Run the typed curl command                      |
| `Ctrl+V`              | Paste from clipboard                            |
| `Ctrl+C`              | Copy last command output                        |

Focus moves through: **Instructions → Solutions → Terminal → Answer → Submit → Back**

---

### Quests

| #   | Quest                      | Folder                                                         |
| --- | -------------------------- | -------------------------------------------------------------- |
| 1   | Day 1: Inventory Check     | [quests/01-Get](quests/01-Get)                                 |
| 2   | Day 2: Adding Items        | [quests/02-Post](quests/02-Post)                               |
| 3   | Day 3: Maintain and Update | [quests/03-Put-Patch-Delete](quests/03-Put-Patch-Delete)       |
| 4   | The Elemental Search       | [quests/04-Query & Encoding](quests/04-Query%20%26%20Encoding) |
| 5   | Payslip Uploader           | [quests/05-File-Upload](quests/05-File-Upload)                 |
| 6   | Strict API Contracts       | [quests/06-Headers](quests/06-Headers)                         |
| 7   | The Manager's Secret       | [quests/07-Header-Inspection](quests/07-Header-Inspection)     |
| 8   | The Galactic Relay         | [quests/08-Status-Codes](quests/08-Status-Codes)               |
| 9   | The Digital Detour         | [quests/09-Redirects](quests/09-Redirects)                     |
| 10  | Identity & Access          | [quests/10-Auth-JWT](quests/10-Auth-JWT)                       |
| 11  | JSON Querying with jq      | [quests/11-JQ](quests/11-JQ)                                   |

---

### How to solve a quest

1. Launch the app and press `Enter` on **Levels**
2. Select a quest and press `Enter` to open it
3. Read the **Instructions** panel on the left
4. Use the **Terminal** panel on the right to run `curl` commands against the local server
5. If the quest asks for an answer, type it in the **Answer** box
6. Tab to **Submit** and press `Enter` to verify the app checks your work against the database
7. If it fails, read the error and try again. The server stays running until you go back.

> Start from Quest 1 and work your way up each quest builds on concepts from the previous ones.

---

### Tips

- **Maximize your terminal**: For the best experience, run your terminal in **full screen**. This ensures all quest content, animations, and the dual-column layout are displayed correctly without clipping.
- **The server is already running** when you open a quest you don't need to start anything manually.
- **Read the instructions fully** before running any command. The quest often tells you the exact endpoint and method to use.
- **If a command produces no output**, the server may still be starting. Wait a second and try again.
- **Disable AI assistance while solving** you'll learn far more by reading the error, checking the curl man page, and trying again.

---

### Troubleshooting

- **`curl-quests: command not found`** after `cargo install` make sure `~/.cargo/bin` is in your `PATH`:

  ```sh
  export PATH="$HOME/.cargo/bin:$PATH"
  ```

  Add that line to your `~/.bashrc` or `~/.zshrc` to make it permanent.

- **`(no output — is the server running?)`** in the terminal the server needs a moment to bind the port. Press `Enter` again to rerun the command.

- **Quest stuck / server not responding** press `Esc` to go back to the quest grid, then re-enter the quest. This restarts the server and reseeds the database.

- **`jq: command not found`** install jq before attempting Quest 11+:
  - macOS: `brew install jq`
  - Ubuntu/Debian: `sudo apt install jq`

---

### Contact

For any issues, contact either:

- [Lite Quests](https://x.com/litequests)
- [Mani Yadla](https://x.com/mani_yadla_)
- [Ananya Pappula](https://x.com/AnanyaPappula)
