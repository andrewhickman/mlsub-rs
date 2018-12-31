pub enum Ty<C> {
    // UnboundVar(V),
    BoundVar(usize),
    Constructed(C),
    Add(Box<Ty<C>>, Box<Ty<C>>),
    Zero,
    Recursive(Box<Ty<C>>),
}