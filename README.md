# breadcrumbs
Breadcrumbs is a beautiful, tiny traceback and logging library for Rust that offers seamless integration with `#![no_std]`, `#[no_panic]` multi-threading and concurrency. 

## Features
- Beautifully-formatted traceback of logs (supporting `Display` and `Debug`)
- Dynamic log levels
- Dynamic log channels
- Seamless integration with `#![no_std]` and `#[no_panic]`
- Multi-threading and concurrent logging with no special syntax
- Easy-to-use macros
- Support for listeners to be notified of new logs

## Usage
Add the following to your `Cargo.toml`:
```toml
[dependencies]
breadcrumbs = "0.1.3"
```

Then, initalize `breadcrumbs` once in your `main.rs` or `lib.rs`:
```rust
use breadcrumbs::init;

init!();
```

You can set a custom log listener with ease by implementing the `LogListener` trait:
```rust
use breadcrumbs::{init, LogListener, Log, LogLevel};
struct MyLogListener;
 
impl LogListener for MyLogListener {
    fn on_log(&mut self, log: Log) {
        if log.level.is_at_least(LogLevel::Warn) {
            println!("{}", log);
        }
    }
}
 
init!(MyLogListener);
```

Then, simply use the `log!` macro from or its variants from anywhere to log messages:
```rust
use breadcrumbs::{log, log_level, log_channel, LogLevel};

// A basic log message
log!("Hello, world!");

// A log message with a custom level
log_level!(LogLevel::Info, "Test log message");

// A log message with a custom channel
log_channel!("test_channel", "Test log message");

// A log message with a custom channel and level
log!(LogLevel::Info, "test_channel", "Test log message");
```

Access a traceback of log messages from anywhere with the `traceback!` macro or its variants:
```rust
use breadcrumbs::{traceback, traceback_channel, traceback_level, LogLevel};

// A basic traceback, fetching all logged messages
let t = traceback!();

// A traceback with a custom channel, fetching messages in this channel
let t = traceback_channel!("my-channel");

// A traceback with a custom level, fetching messages of this level or higher
let traceback = traceback_level!(LogLevel::Warn);

// A traceback with a custom channel and level, fetching messages in this channel of this level or higher
let traceback = traceback!(LogLevel::Warn, "test_channel");
```

`Traceback` and `Log` objects beautifully implement `Display` and `Debug`:
```rust
use breadcrumbs::traceback;

let t = traceback!();
println!("{}", t);
pritnln!("{:?}", t);
```

## Example

```rust
use breadcrumbs::{init, log, log_level, log_channel, traceback, LogLevel, LogListener, Log};

struct MyLogListner;

impl LogListener for MyLogListner {
    fn on_log(&mut self, log: Log) {
        if log.level.is_at_least(LogLevel::Warn) {
            println!("{}", log);
        }
    }
}

fn main() {
    init!(MyLogListner);

    log!("Hello, world!");
    log_level!(LogLevel::Info, "Test log message");
    log_channel!("test_channel", "Test log message");
    log!(LogLevel::Warn, "test_channel", "Test log message");
}
```

## `#![no_std]` Example Usecase

```rust
#![no_std]

use breadcrumbs::{init, log, log_level, log_channel, traceback, LogLevel, Log, LogListener};
use your_crate::serial_println;

struct MyLogListner;

impl LogListener for MyLogListner {
    fn on_log(&mut self, log: Log) {
        if log.level.is_at_least(LogLevel::Warn) {
            serial_println!("{}", log);
        }
    }
}

fn main() {
    init!(MyLogListner);

    log!("Hello, world!");
    log_level!(LogLevel::Info, "Test log message");
    log_channel!("test_channel", "Test log message");
    log!(LogLevel::Warn, "test_channel", "Test log message");
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log!(LogLevel::Critical, "PANIC!");

    let t = traceback!();
    serial_println!("{}", t);

    loop {}
}
```
