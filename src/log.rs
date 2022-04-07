#[allow(unused_imports)]
use console::style;

macro_rules! success {
    ($fmt:expr) => {
        println!("{}", style(format!("✔ {}", $fmt)).green());
    };
    ($fmt:expr $(, $($arg:tt)*)?) => {
        println!("{}", style(format!(concat!("✔ ", $fmt), $($($arg)*)?)).green());
    };
}

macro_rules! info {
    ($fmt:expr) => {
        println!("{}", style(format!("🛈 {}", $fmt)).blue());
    };
    ($fmt:expr $(, $($arg:tt)*)?) => {
        println!("{}", style(format!(concat!("🛈 ", $fmt), $($($arg)*)?)).blue());
    };
}

macro_rules! warn {
    ($fmt:expr) => {
        println!("{}", style(format!("⚠ {}", $fmt)).yellow());
    };
    ($fmt:expr $(, $($arg:tt)*)?) => {
        println!("{}", style(format!(concat!("⚠ ", $fmt), $($($arg)*)?)).yellow());
    };
}

macro_rules! error {
    ($fmt:expr) => {
        println!("{}", style(format!("⚠ {}", $fmt)).red());
    };
    ($fmt:expr $(, $($arg:tt)*)?) => {
        println!("{}", style(format!(concat!("⚠ ", $fmt), $($($arg)*)?)).red());
    };
}
