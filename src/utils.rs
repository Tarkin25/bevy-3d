pub trait ToUsize {
    fn to_usize(self) -> usize;
}

impl ToUsize for isize {
    fn to_usize(self) -> usize {
        self.try_into().unwrap()
    }
}

pub trait ToIsize {
    fn to_isize(self) -> usize;
}

impl ToIsize for usize {
    fn to_isize(self) -> usize {
        self.try_into().unwrap()
    }
}
