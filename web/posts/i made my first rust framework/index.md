# I Made My First Rust Framework
_First of many, maybe_

Two days ago (December 31st, 2025) I decided to make my first Rust framework.

The reason as to why was because I had simply just started making more TUI SSH apps and found myself writing a lot of boilerplate code over and over.

So, to make this easier, I decide to abstract (nearly) all of it away. Inspired by how charmbracelet made it so simple to host [bubbletea](https://github.com/charmbracelet/bubbletea) apps on an SSH server, I created the [chai](https://github.com/kllarena07/chai-framework) framework, powered by [ratatui](https://github.com/ratatui/ratatui) and [russh](https://github.com/Eugeny/russh/).

The design philosophy of the framework was to allow the user to focus on writing their ratatui apps rather than configuring both their ssh server and their code to be able to import it into their server, which was something that I was doing often. Using the chai framework is as easy as simply [turbofishing](https://rust.code-maven.com/turbofish) your app into the `ChaiServer` and running it.
```rust
mod app;
use app::MyApp; // your TUI program
use chai_framework::{ChaiApp, ChaiServer, load_host_keys};

#[tokio::main]
async fn main() {
    let host_key = load_system_host_keys("id_ed25519");
    let config = Config {
        // server config here
        keys: vec![host_key],
    };

    let mut server = ChaiServer::<MyApp>::new(2222);
    server.run(config).await.expect("Failed running server");
}
```
Currently I am experimenting with using procedural macros to simplify this process even further, and I’m excited to see what they can do. Even more so, I’m excited to see if people will find this framework useful.

_Written with ❤️ by Krayon_  
Follow me: [x.com/krayon](https://x.com/krayondev)

