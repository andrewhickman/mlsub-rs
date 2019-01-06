#[derive(Debug, Clone)]
pub enum Ty<C, V> {
    UnboundVar(V),
    BoundVar(usize),
    Constructed(C),
    Add(Box<Ty<C, V>>, Box<Ty<C, V>>),
    Zero,
    Recursive(Box<Ty<C, V>>),
}
