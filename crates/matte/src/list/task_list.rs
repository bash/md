use crate::prefix::Prefix;
use crate::Events;
use fmtastic::BallotBox;
use pulldown_cmark::Event;

#[derive(Debug)]
pub(super) struct TaskListMarker(pub(super) bool);

impl TaskListMarker {
    pub(super) fn try_consume(events: Events) -> Option<Self> {
        let mut events = events.lookahead();
        if let Event::TaskListMarker(checked) = events.next()? {
            _ = events.commit();
            Some(Self(checked))
        } else {
            None
        }
    }

    pub(super) fn to_prefix(&self) -> Prefix {
        Prefix::continued(format!("{:#} ", BallotBox(self.0)))
    }
}
