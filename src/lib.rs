#![no_std]

//! # breadcrumbs
//! Breadcrumbs is a beautiful, tiny traceback and logging library for Rust that offers seamless integration with `#![no_std]`, `#[no_panic]` multi-threading and concurrency. 
//! 
//! ## Features
//! - Beautifully-formatted traceback of logs (supporting `Display` and `Debug`)
//! - Dynamic log levels
//! - Dynamic log channels
//! - Seamless integration with `#![no_std]` and `#[no_panic]`
//! - Multi-threading and concurrent logging with no special syntax
//! - Easy-to-use macros
//! - Support for listeners to be notified of new logs

// Import the necessary crates
extern crate alloc;
use alloc::{
    vec::Vec,
    sync::Arc,
    boxed::Box,
    string::String,
    format
};
use lazy_static::lazy_static;
use spin::Mutex;

/// Enum representing different log levels.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum LogLevel {
    Verbose,
    Info,
    Warn,
    Error,
    Critical,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl core::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let level_str = match self {
            LogLevel::Verbose => "Verbose",
            LogLevel::Info => "Info",
            LogLevel::Warn => "Warn",
            LogLevel::Error => "Error",
            LogLevel::Critical => "Critical",
        };
        write!(f, "{}", level_str)
    }
}
impl LogLevel {
    /// Checks if the current log level is at least as severe as the provided level.
    /// ```rust
    /// use breadcrumbs::LogLevel;
    /// let log_level = LogLevel::Info;
    /// assert!(log_level.is_at_least(LogLevel::Info));
    /// assert!(log_level.is_at_least(LogLevel::Verbose));
    /// assert!(!log_level.is_at_least(LogLevel::Warn));
    /// ```
    pub fn is_at_least(&self, level: LogLevel) -> bool {
        match level {
            LogLevel::Verbose => true,
            LogLevel::Info => self != &LogLevel::Verbose,
            LogLevel::Warn => self != &LogLevel::Verbose && self != &LogLevel::Info,
            LogLevel::Error => self != &LogLevel::Verbose && self != &LogLevel::Info && self != &LogLevel::Warn,
            LogLevel::Critical => self == &LogLevel::Critical,
        }
    }

    pub fn from_str(level: &str) -> LogLevel {
        match level {
            "Verbose" => LogLevel::Verbose,
            "Info" => LogLevel::Info,
            "Warn" => LogLevel::Warn,
            "Error" => LogLevel::Error,
            "Critical" => LogLevel::Critical,
            _ => LogLevel::Info,
        }
    }
}

/// Represents a log entry.
/// `Log` beautifully implements `Display` for easy printing.
/// ```rust
/// use breadcrumbs::Log;
/// let log = Log::new(String::from("test_channel"), breadcrumbs::LogLevel::Info, String::from("Test log message"));
/// assert_eq!(format!("{}", log), "[test_channel/Info] Test log message");
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Log {
    pub channel: String,
    pub level: LogLevel,
    pub message: String,
}

impl core::fmt::Display for Log {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.channel != "" {
            return write!(f, "[{}/{}] {}", self.channel, self.level, self.message);
        } else {
            return write!(f, "[{}] {}", self.level, self.message)
        }
    }
}

impl Log {
    /// Creates a new log entry.
    pub fn new(channel: String, level: LogLevel, message: String) -> Log {
        Log {
            channel,
            level,
            message,
        }
    }
    /// Removes the log from the stored traceback of logs.
    /// This log will not use up memory or be printed by the traceback macros.
    /// Useful in embedded systems where memory is limited.
    /// 
    /// # Example Usecase
    /// ```rust
    /// use breadcrumbs::{LogListener, Log};
    /// 
    /// struct MyLogListener;
    /// impl LogListener for MyLogListener {
    ///     fn on_log(&mut self, log: Log) {
    ///         if !log.level.is_at_least(breadcrumbs::LogLevel::Warn) {
    ///             log.remove();
    ///         }
    ///     }
    /// }
    /// ```
    pub fn remove(&self) {
        let mut logs = LOGS.lock();
        let index = logs.iter().position(|log| log == self).unwrap();
        logs.remove(index);
    }
}

/// A trait for handling log entries.
pub trait LogListener: Send + Sync {
    fn on_log(&mut self, log: Log);
}

lazy_static! {
    static ref LOGS: Arc<Mutex<Vec<Log>>> = Arc::new(Mutex::new(Vec::new()));
    static ref LOG_LISTENER: Arc<Mutex<Option<Box<dyn LogListener>>>> = Arc::new(Mutex::new(None));
}

/// Initializes the logging system without a listener.
/// Note that the `init!` macro is the preferred method to do this in the public API.
/// ```rust
/// use breadcrumbs::init;
/// init();
/// ```
pub fn init() {
    LOGS.lock().clear();
    *LOG_LISTENER.lock() = None;
}

/// Initializes the logging system with a listener.
/// Note that the `init!` macro is the preferred method to do this in the public API.
/// ```rust
/// use breadcrumbs::{init_with_listener, LogListener};
/// struct MyLogListener;
/// 
/// impl LogListener for MyLogListener {
///    fn on_log(&mut self, log: breadcrumbs::Log) {
///       println!("{}", log);
///   }
/// }
/// 
/// init_with_listener(Box::new(MyLogListener));
/// ```
pub fn init_with_listener(listener: Box<dyn LogListener>) {
    LOGS.lock().clear();
    *LOG_LISTENER.lock() = Some(listener);
}


/// A macro for initializing the logging system.
/// 
/// # Use
/// 
/// To initialize the logging system without a listener, do not pass any arguments.
/// 
/// To initialize the logging system with a listener, pass a listener implementing `LogListener` as the first argument.
/// 
/// # Examples
/// 
/// Initialize the logging system without a listener:
/// ```
/// use breadcrumbs::init;
/// init!();
/// ```
/// 
/// Initialize the logging system with a listener:
/// ```
/// use breadcrumbs::{init, LogListener};
/// struct MyLogListener;
/// 
/// impl LogListener for MyLogListener {
///   fn on_log(&mut self, log: breadcrumbs::Log) {
///      println!("{}", log);
///   }
/// }
/// 
/// init!(MyLogListener);
#[macro_export]
macro_rules! init {
    () => {
        $crate::init()
    };
    ($arg1:expr) => {
        extern crate alloc;
        use alloc::boxed::Box;
        $crate::init_with_listener(Box::new($arg1))
    };
}

/// Logs a message with an optional log level and channel. 
/// Note that the `log!` macro is the preferred method to do this in the public API.
/// ```rust
/// use breadcrumbs::{log, LogLevel};
/// log(Some(LogLevel::Info), Some(String::from("test_channel")), String::from("Test log message"));
/// ```
pub fn log(level: Option<LogLevel>, channel: Option<String>, message: String) {
    let log = Log::new(channel.unwrap_or(String::from("")), level.unwrap_or(LogLevel::Info), message.clone());
    LOGS.lock().push(log.clone());
    if let Some(listener) = &mut *LOG_LISTENER.lock() {
        listener.on_log(Log::new(log.channel, log.level, log.message));
    }
}

/// Represents a traceback of logs.
/// `Traceback` beautifully implements `Display` for easy printing.
/// ```rust
/// use breadcrumbs::{Traceback, Log};
/// let traceback = Traceback(vec![Log::new(String::from("test_channel"), breadcrumbs::LogLevel::Info, String::from("Test log message"))]);
/// assert_eq!(format!("{}", traceback), "[test_channel/Info] Test log message\n");
/// ```
pub struct Traceback(pub Vec<Log>);

impl Traceback {
    /// Converts the traceback to a beautifully-formatted string.
    /// ```rust
    /// use breadcrumbs::traceback;
    /// let traceback = traceback!();
    /// let traceback_string = traceback.to_string();
    /// ```
    pub fn to_string(&self) -> String {
        let mut traceback = String::new();
        for log in &self.0 {
            traceback.push_str(&format!("{}\n", log));
        }
        traceback
    }
}

impl core::fmt::Display for Traceback {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Retrieves a traceback of logs based on the minimum log level and channel filter.
/// Note that the `traceback!` macro is the preferred method to do this in the public API.
/// ```rust
/// use breadcrumbs::{get_logs_traceback, LogLevel};
/// let traceback = get_logs_traceback(Some(LogLevel::Warn), Some(vec![String::from("test_channel")]));
/// ```
pub fn get_logs_traceback(min_level: Option<LogLevel>, channels: Option<Vec<String>>) -> Traceback {
    let mut logs = Vec::new();
    for log in LOGS.lock().iter() {
        if min_level.is_some() && !log.level.is_at_least(min_level.unwrap()) {
            continue;
        }
        if channels.is_some() && !channels.as_ref().unwrap().contains(&log.channel) {
            continue;
        }
        logs.push(log.clone());
    }
    Traceback(logs)
}

/// A macro for generating a `Traceback` of logs, optionally filtered by log level and channel.
/// 
/// To only specify a `LogLevel`, use the `traceback_level!` macro.
/// 
/// To only specify a `channel`, use the `traceback_channel!` macro.
/// 
/// # Examples
/// 
/// Traceback with default values:
/// 
/// ```
/// use breadcrumbs::traceback;
/// let traceback = traceback!();
/// ```
/// 
/// Traceback with a custom log level and channel:
/// 
/// ```
/// use breadcrumbs::{traceback, LogLevel};
/// let traceback = traceback!(LogLevel::Warn, "test_channel");
/// ```
#[macro_export]
macro_rules! traceback {
    () => {
        $crate::get_logs_traceback(None, None)
    };
    ($arg1:expr, $arg2:expr) => {
        $crate::get_logs_traceback(Some($arg1), Some(vec![$arg2.to_string()]))
    };
}

/// A macro for generating a `Traceback` of logs given only a log level.
/// 
/// # Examples
/// 
/// Basic usage:
/// 
/// ```
/// use breadcrumbs::{traceback_level, LogLevel};
/// let traceback = traceback_level!(LogLevel::Warn);
/// ```
#[macro_export]
macro_rules! traceback_level {
    ($arg1:expr) => {
        $crate::get_logs_traceback(Some($arg1), None)
    };
}

/// A macro for generating a `Traceback` of logs given only a channel.
/// 
/// # Examples
/// 
/// Basic usage:
/// 
/// ```
/// use breadcrumbs::traceback_channel;
/// let traceback = traceback_channel!("test_channel");
/// ```
#[macro_export]
macro_rules! traceback_channel {
    ($arg1:expr) => {
        $crate::get_logs_traceback(None, Some(vec![$arg1.to_string()]))
    };
}



/// A macro for logging messages with an optional log level and channel.
/// 
/// To only specify a `LogLevel`, use the `log_level!` macro.
/// 
/// To only specify a `channel`, use the `log_channel!` macro.
/// 
/// # Examples
/// 
/// Log with a log level, channel and message
/// ```rust
/// use breadcrumbs::{log, LogLevel};
/// log!(LogLevel::Info, "test_channel", "Test log message");
/// ```
/// 
/// Log with just a message
/// 
/// ```rust
/// use breadcrumbs::log;
/// log!("Test log message");
/// ```
#[macro_export]
macro_rules! log {
    ($arg1:expr, $arg2:expr, $arg3:expr) => {
        $crate::log(Some($arg1), Some($arg2.to_string()), $arg3.to_string())
    };
    ($arg1:expr) => {
        $crate::log(None, None, $arg1.to_string())
    };
}

/// A macro for logging messages with a log level only.
/// 
/// # Examples
/// 
/// ```rust
/// use breadcrumbs::{log_level, LogLevel};
/// log_level!(LogLevel::Info, "Test log message");
/// ```
#[macro_export]
macro_rules! log_level {
    ($arg1:expr, $arg2:expr) => {
        $crate::log(Some($arg1), None, $arg2.to_string())
    };
}

/// A macro for logging messages with a channel only.
/// 
/// # Examples
/// 
/// ```rust
/// use breadcrumbs::{log_channel, LogLevel};
/// log_channel!("test_channel", "Test log message");
/// ```
#[macro_export]
macro_rules! log_channel {
    ($arg1:expr, $arg2:expr) => {
        $crate::log(None, Some($arg1.to_string()), $arg2.to_string())
    };
}



#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use crate::alloc::string::ToString;
    
    // Test the LogLevel enum
    #[test]
    fn test_log_level_enum() {
        assert_eq!(LogLevel::from_str("Verbose"), LogLevel::Verbose);
        assert_eq!(LogLevel::from_str("Info"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("Warn"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("Error"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("Critical"), LogLevel::Critical);
    }

    // Test Log and LogListener
    struct MockLogListener {
        received_log: Option<Log>,
    }

    impl MockLogListener {
        fn new() -> Self {
            MockLogListener { received_log: None }
        }
    }

    impl LogListener for MockLogListener {
        fn on_log(&mut self, log: Log) {
            self.received_log = Some(log);
        }
    }

    // Wrapper struct that implements LogListener for Arc<Mutex<MockLogListener>>
    struct MockLogListenerWrapper(Arc<Mutex<MockLogListener>>);

    impl LogListener for MockLogListenerWrapper {
        fn on_log(&mut self, log: Log) {
            self.0.lock().on_log(log);
        }
    }

    #[test]
    fn test_log_creation_and_handling() {
        let mock_listener = Arc::new(Mutex::new(MockLogListener::new()));
        let mock_listener_wrapper = MockLogListenerWrapper(mock_listener.clone());
        init!(mock_listener_wrapper);

        log!(LogLevel::Info, "test_channel", "Test log message");

        let received_log = mock_listener.lock().received_log.clone().expect("Log not received by listener");
        assert_eq!(received_log.level, LogLevel::Info);
        assert_eq!(received_log.channel, "test_channel");
        assert_eq!(received_log.message, "Test log message");
    }

    // Test traceback generation
    #[test]
    fn test_traceback_generation() {
        log!(LogLevel::Info, "channel1", "Log 1");
        log!(LogLevel::Warn, "channel2", "Log 2");
        log!(LogLevel::Error, "channel1", "Log 3");

        let traceback = traceback!(LogLevel::Warn, "channel2").to_string();
        assert!(traceback.contains("[channel2/Warn] Log 2"));
        assert!(!traceback.contains("[channel1/Info] Log 1"));
        assert!(!traceback.contains("[channel1/Error] Log 3"));
    }

    // Test log macros
    #[test]
    fn test_log_macros() {
        log!(LogLevel::Info, "test_channel", "Test log message");
        log_level!(LogLevel::Info, "Test log message");
        log_channel!("test_channel", "Test log message 2");

        let traceback = traceback!().to_string();
        assert!(traceback.contains("[test_channel/Info] Test log message 2"));
        assert!(traceback.contains("[test_channel/Info] Test log message "));
        assert!(traceback.contains("[Info] Test log message"));
    }

    // Test the example in the README
    #[test]
    fn read_me_example() {
        init!();

        log!("Hello, world!");
        log_level!(LogLevel::Info, "Test log message");
        log_channel!("test_channel", "Test log message");
        log!(LogLevel::Info, "test_channel", "Test log message");
    }

    struct MyLogListener2 {
        success: bool,
    }

    impl LogListener for MyLogListener2 {
        fn on_log(&mut self, log: Log) {
            if log.level.is_at_least(LogLevel::Warn) {
                self.success = true;
            }
        }
    }

    struct MockLogListenerWrapper2(Arc<Mutex<MyLogListener2>>);

    impl LogListener for MockLogListenerWrapper2 {
        fn on_log(&mut self, log: Log) {
            self.0.lock().on_log(log);
        }
    }

    #[test]
    fn no_std_readme_example() {
        let log_handler = Arc::new(Mutex::new(MyLogListener2 { success: false }));
        let log_handler_wrapper = MockLogListenerWrapper2(log_handler.clone());

        init!(log_handler_wrapper);

        log!(LogLevel::Error, "test_channel", "Test log message");

        assert!(log_handler.lock().success);
    }
}

