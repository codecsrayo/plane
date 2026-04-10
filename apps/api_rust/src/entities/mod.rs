pub mod issue;
pub mod project;
pub mod state;
pub mod workspace;

pub mod prelude {
    pub use super::issue::Entity as Issue;
    pub use super::project::Entity as Project;
    pub use super::state::Entity as State;
    pub use super::workspace::Entity as Workspace;
}
