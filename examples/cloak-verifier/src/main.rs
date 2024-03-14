use prusti_contracts::*;

pub struct LocalChannelApp<S, M> where
    S: LocalChannelState,
    M: SharedMemoryState
{
    id: usize,
    shared_memory: SharedMemory<M>,
    state: S,  // (0) Created --> (1) Connected --> (2) Received --> (3) Established
}

#[derive(Clone, Copy)]
pub struct SharedMemory<M> where
    M: SharedMemoryState
{
    ipa: usize,
    state: M,  // (0) Unmapped --> (1) ReadOnly --> (2) ReadWrite
}
impl SharedMemory<Unmapped> {
    #[ensures((&result).state.is_unmapped())]
    pub const fn new(ipa: usize) -> SharedMemory<Unmapped> {
        Self {
            ipa: ipa,
            state: Unmapped,
        }
    }
}
impl SharedMemory<Unmapped> {
    #[requires(self.state.is_unmapped())]
    #[ensures((&result).state.is_read_only() && old(self).ipa == (&result).ipa)]
    pub fn into_read_only(self) -> SharedMemory<ReadOnly> {
        SharedMemory {
            ipa: self.ipa,
            state: ReadOnly,
        }
    }
}
impl SharedMemory<ReadOnly> {
    #[requires(self.state.is_read_only())]
    #[ensures((&result).state.is_read_write() && old(self).ipa == (&result).ipa)]
    pub fn into_read_write(self) -> SharedMemory<ReadWrite> {
        SharedMemory {
            ipa: self.ipa,
            state: ReadWrite,
        }
    }
}

impl<S: LocalChannelState, M: SharedMemoryState> LocalChannelApp<S, M> {
    #[ensures( ((&result).state.is_created()) && ((&result).shared_memory.state.is_unmapped()) )]
    pub const fn new() -> LocalChannelApp<Created, Unmapped> {
        LocalChannelApp {
            id: 0,
            shared_memory: SharedMemory::<Unmapped>::new(0),
            state: Created,
        }
    }
}

impl LocalChannelApp<Created, Unmapped> {
    #[requires( ((&self).state.is_created()) && ((&self).shared_memory.state.is_unmapped()) )]
    #[ensures( ((&result).state.is_connected()) && ((&result).shared_memory.state.is_read_only()) )]
    pub fn connect(self) -> LocalChannelApp<Connected, ReadOnly> {
        // do something here!

        LocalChannelApp {
            id: self.id,
            shared_memory: self.shared_memory.into_read_only(),
            state: Connected,
        }
    }
}

impl LocalChannelApp<Connected, ReadOnly> {
    #[requires( ((&self).state.is_connected()) && ((&self).shared_memory.state.is_read_only()) )]
    #[ensures( ((&result).state.is_received()) && ((&result).shared_memory.state.is_read_only()) )]
    pub fn wait_for_signed_cert(self) -> LocalChannelApp<Received, ReadOnly> {
        // do something here!
        
        LocalChannelApp {
            id: self.id,
            shared_memory: self.shared_memory,
            state: Received,
        }
    }
}

impl LocalChannelApp<Received, ReadOnly> {
    #[requires( ((&self).state.is_received()) && ((&self).shared_memory.state.is_read_only()) )]
    #[ensures( ((&result).state.is_established()) && ((&result).shared_memory.state.is_read_write()) )]
    pub fn check_signed_cert(self) -> LocalChannelApp<Established, ReadWrite> {
        // do something here!
        
        LocalChannelApp {
            id: self.id,
            shared_memory: self.shared_memory.into_read_write(),
            state: Established,
        }
    }
}

impl LocalChannelApp<Established, ReadWrite> {
    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) )]
    pub fn perform(&self) {
        //println!("perform! {}", self.id);
    }
}

// Typestate definitions for LocalChannelState
pub struct Start;
pub struct Created;
pub struct Connected;
pub struct Received;
pub struct Established;

pub trait LocalChannelState {
    #[pure]
    fn is_start(&self) -> bool { false }

    #[pure]
    fn is_created(&self) -> bool { false }

    #[pure]
    fn is_connected(&self) -> bool { false }

    #[pure]
    fn is_received(&self) -> bool { false }

    #[pure]
    fn is_established(&self) -> bool { false }
}

#[refine_trait_spec]
impl LocalChannelState for Start {
    #[pure]
    #[ensures(result == true)]
    fn is_start(&self) -> bool { true }
}

#[refine_trait_spec]
impl LocalChannelState for Created {
    #[pure]
    #[ensures(result == true)]
    fn is_created(&self) -> bool { true }
}

#[refine_trait_spec]
impl LocalChannelState for Connected {
    #[pure]
    #[ensures(result == true)]
    fn is_connected(&self) -> bool { true }
}

#[refine_trait_spec]
impl LocalChannelState for Received {
    #[pure]
    #[ensures(result == true)]
    fn is_received(&self) -> bool { true }
}

#[refine_trait_spec]
impl LocalChannelState for Established {
    #[pure]
    #[ensures(result == true)]
    fn is_established(&self) -> bool { true }
}

// Typestate definitions for SharedMemoryState
pub struct Unmapped;
pub struct ReadOnly;
pub struct ReadWrite;

pub trait SharedMemoryState {
    #[pure]
    fn is_unmapped(&self) -> bool { false }

    #[pure]
    fn is_read_only(&self) -> bool { false }

    #[pure]
    fn is_read_write(&self) -> bool { false }
}

#[refine_trait_spec]
impl SharedMemoryState for Unmapped {
    #[pure]
    #[ensures(result == true)]
    fn is_unmapped(&self) -> bool { true }
}

#[refine_trait_spec]
impl SharedMemoryState for ReadOnly {
    #[pure]
    #[ensures(result == true)]
    fn is_read_only(&self) -> bool { true }
}

#[refine_trait_spec]
impl SharedMemoryState for ReadWrite {
    #[pure]
    #[ensures(result == true)]
    fn is_read_write(&self) -> bool { true }
}

fn test_static_state_transition() {
    let channel = LocalChannelApp::<Start, Unmapped>::new();
    let channel = channel.connect();
    let channel = channel.wait_for_signed_cert();
    let channel = channel.check_signed_cert();
    channel.perform();
}

fn main() {
    test_static_state_transition();
}