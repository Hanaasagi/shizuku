#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct SrcSpan {
    pub start: u32,
    pub end: u32,
}
