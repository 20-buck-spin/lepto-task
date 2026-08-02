#[derive(Clone, PartialEq, Eq)]
pub enum LoadingState<T> {
    Loading,
    Empty,
    Ready(T),
}
