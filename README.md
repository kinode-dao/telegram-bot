# Telegram Bot

Provides an API and a process to build telegram bots on kinode.

## Example Usage

First, you need to obtain a bot token from BotFather on telegram.
Once you have that, you can clone this repo, run `kit b` on it to build it, (instructions for installing kit [here](https://github.com/kinode-dao/kit)), and put it's wasm in your package folder as `tg.wasm`.

Then put [`tg/src/tg_api.rs`](https://github.com/kinode-dao/command_center/blob/jurij/command_center/src/tg_api.rs) somewhere in your app, and make it callable by putting `mod tg_api` in your code.

You might have to add some dependencies used by the bot to your `Cargo.toml`:

```rust
frankenstein = { version = "0.30", default-features = false, features = ["telegram-trait"] }
url = "2.5.0"
```

Then, spawning a worker that forwards you updates:

```rust
let (api, tg_worker) = tg_api::init_tg_bot(our.clone, "your_token", None)?;
// the third argument is an optional getUpdatesParams, here you can specify if you want specific updates only!
```

Updates will come in from tg_worker processId with the enum:

```rust
pub enum TgResponse {
    Update(TgUpdate),
    Error(String),
}

pub struct TgUpdate {
    pub updates: Vec<Update>,
}
```

And calling a method like `get_chat_member_count`:

```rust
use frankenstein::{GetChatMemberCountParams, ChatId}
let params = frankenstein::GetChatMemberCountParams {
    chat_id: ChatId::Integer(123)
}
let member_count: u32 = api.get_chat_member_count(&params)?;
```

## Repos

Some projects using this:

- [simple hello bot](https://github.com/bitful-pannul/hellobot)
- [token gated chat](https://github.com/bitful-pannul/kinogate?tab=readme-ov-file)
- [trader bot](https://github.com/bitful-pannul/trader)
