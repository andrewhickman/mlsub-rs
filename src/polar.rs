#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty<C, V> {
    Zero,
    Add(Box<Ty<C, V>>, Box<Ty<C, V>>),
    UnboundVar(V),
    BoundVar(usize),
    Constructed(C),
    Recursive(Box<Ty<C, V>>),
}
