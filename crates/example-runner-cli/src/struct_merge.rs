pub trait StructMerge {
    fn join_inplace(&mut self, other: Self);
}
