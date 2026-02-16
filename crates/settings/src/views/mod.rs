pub mod about;
pub mod ai;
pub mod appearance;
pub mod general;
pub mod meshy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    General,
    Appearance,
    Ai,
    Meshy,
    About,
}
