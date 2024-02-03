pub trait StructMerge {
    /// Joins two `ExampleConfig`'s by keeping existing values from `self`.
    fn join(self, other: Self) -> Self;

    fn merge(self, other: Self) -> Self
    where
        Self: Sized,
    {
        other.join(self)
    }
}
