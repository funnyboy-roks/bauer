pub mod state;

pub mod sealed {
    /// This is not _technically_ sealed, but a used would have to manually implement this trait,
    /// so it's good enough
    pub trait Sealed {}
}
