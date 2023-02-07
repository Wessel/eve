use crate::util::types::Command;

// pub mod meta;
pub mod stablediffusion;
pub mod general;

pub fn prepare() -> Vec<Command> {
    vec![
      stablediffusion::imagine(),
      stablediffusion::show(),
      stablediffusion::report(),
      general::help(),
      general::ping()
    ]
}