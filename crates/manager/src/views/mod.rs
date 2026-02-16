pub mod create_pet;
pub mod generation;
pub mod pet_list;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    PetList,
    CreatePet,
    Generation,
}
