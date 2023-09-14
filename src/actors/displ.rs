use std::fmt::Write;

use indicatif::{ProgressStyle, ProgressBar, ProgressState};
use tiny_tokio_actor::{Actor, Message, Handler, async_trait, ActorContext};

use super::event::Event;

/// --------------------------- ///
/// ---------- Actor ---------- ///
/// --------------------------- ///
///
/// Display actor
#[derive(Clone)]
pub struct Displ {
    pub spinner_style: ProgressStyle,
    pub progress_bar: Option<ProgressBar>,
}

/// Display actor implementation.
impl Actor<Event> for Displ {}

/// Default implementation for the display actor.
impl Default for Displ {
    fn default() -> Self {
        Displ {
            spinner_style :ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}").unwrap().tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
            progress_bar: None,
        }
    }
}

/// --------------------------- ///
/// -------- Messages --------- ///
/// --------------------------- ///
///
/// Message that allows to display a message to the user.
#[derive(Clone)]
pub struct DisplayMessage {
    pub message: String
}

/// Message implementation for the DisplayMessage message.
impl Message for DisplayMessage {
    /// The type of the result.
    /// A result that contains either the id of the project or an error.
    type Response = Result<(), std::io::Error>;
}

/// Handler for the DisplayMessage message.
#[async_trait]
impl Handler<Event, DisplayMessage> for Displ {
    async fn handle(&mut self, msg: DisplayMessage, _: &mut ActorContext<Event>) -> Result<(), std::io::Error> {
        if let Some(pb) = &self.progress_bar {
            pb.finish_and_clear();
            self.progress_bar = None;
        }
        println!("{}", msg.message);
        Ok(())
    }
}

/// Message that allows to initialize the progress bar. 
#[derive(Clone)]
pub struct InitProgressBar {
    pub message: String,
    pub length: u64,
}

/// Message implementation for the InitProgressBar message.
impl Message for InitProgressBar {
    /// The type of the result.
    type Response = ();
}

/// Handler for the InitProgressBar message.
#[async_trait]
impl Handler<Event, InitProgressBar> for Displ {
    async fn handle(&mut self, msg: InitProgressBar, _: &mut ActorContext<Event>) -> () {
        if let Some(pb) = &self.progress_bar {
            pb.finish_and_clear();
        }
        let new_progress = ProgressBar::new(msg.length);
        new_progress.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

        self.progress_bar = Some(new_progress);
        self.progress_bar.as_mut().unwrap().set_message(msg.message);
    }
}

/// Message that allows to update the progress bar.
#[derive(Clone)]
pub struct IncreaseProgress {
    pub message: String,
}

/// Message implementation for the IncreaseProgress message.
impl Message for IncreaseProgress {
    /// The type of the result.
    /// A result that contains either the id of the project or an error.
    type Response = ();
}

/// Handler for the IncreaseProgress message.
#[async_trait]
impl Handler<Event, IncreaseProgress> for Displ {
    async fn handle(&mut self, msg: IncreaseProgress, _: &mut ActorContext<Event>) -> () {
        if let Some(pb) = &self.progress_bar {
            pb.set_message(msg.message);
            pb.inc(1);
        }
    }
}

