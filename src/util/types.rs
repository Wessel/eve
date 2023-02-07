use crate::structures;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Command = poise::Command<structures::GlobalData, Error>;
pub type Context<'a> = poise::Context<'a, structures::GlobalData, Error>;

#[allow(dead_code)]
pub fn print_type_of<T>(_: &T) {
  println!("{}", std::any::type_name::<T>())
}