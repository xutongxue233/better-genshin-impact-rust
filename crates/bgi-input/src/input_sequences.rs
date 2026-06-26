#[path = "input_sequences_dispatch.rs"]
mod input_sequences_dispatch;
#[path = "input_sequences_input.rs"]
mod input_sequences_input;
#[path = "input_sequences_post_message.rs"]
mod input_sequences_post_message;

pub use input_sequences_dispatch::{InputCancellationToken, InputDispatchReport};
pub use input_sequences_input::{
    release_all_keys_sequence, release_pressed_keys_sequence, InputSequence,
    DEFAULT_RELEASE_MOUSE_BUTTONS,
};
pub use input_sequences_post_message::{make_lparam, PostMessageSequence};
