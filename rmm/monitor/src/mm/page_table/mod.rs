pub trait Level {
    const THIS_LEVEL: usize;
}

pub trait HasSubtable: Level {
    type NextLevel;
}
