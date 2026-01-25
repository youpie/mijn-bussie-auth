pub mod change_information;
pub mod create_instance;

pub trait Client {
    /// Censor sensitive data
    ///
    /// You need to select which attribute you want to be kept
    ///
    /// Zero-trust
    fn censor(self) -> Self;
}
