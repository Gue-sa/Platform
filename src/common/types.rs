use crate::common::utils::*;


#[derive(Clone, Copy)]
pub enum Channel {
    C87B,
    C88B,
    GPS,
    Any
}


#[derive(Clone, Debug)]
pub enum CSTypes {
    SOTDMA,
    ITDMA
}
