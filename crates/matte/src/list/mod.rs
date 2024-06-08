use crate::block::prelude::*;
use counter_style::CounterStyle;

mod bullets;
pub(crate) use bullets::*;
use item::render_item;
mod counter_style;
mod item;
mod task_list;

pub(crate) struct List {
    pub(crate) first_item_number: Option<u64>,
}

impl Block for List {
    fn kind(&self) -> BlockKind {
        BlockKind::List
    }

    fn render<'e>(
        self,
        events: &mut impl Events<'e>,
        ctx: &Context<'_, 'e, '_>,
        w: &mut dyn Write,
    ) -> io::Result<()> {
        let mut counter = CounterStyle::from_context(self.first_item_number, ctx);

        terminated_for! {
            for event in terminated!(events, Event::End(TagEnd::List(..))) {
                reachable! {
                    let Event::Start(Tag::Item) = event {
                        render_item(&counter, events, ctx, w)?;
                        counter.next();
                    }
                }
            }
        }

        Ok(())
    }
}
