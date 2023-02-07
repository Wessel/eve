//TODO: Add command to show styles

mod imagine;
mod report;
mod show;

pub use imagine::execute as imagine;
pub use report::execute as report;
pub use show::execute as show;