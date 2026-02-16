pub mod create_pet;
pub mod pet_list;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    PetList,
    CreatePet,
}
