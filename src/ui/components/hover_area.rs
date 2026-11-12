use cosmic::iced::core::layout;
use cosmic::iced::core::mouse;
use cosmic::iced::core::overlay;
use cosmic::iced::core::renderer;
use cosmic::iced::core::widget::{tree, Operation, Tree};
use cosmic::iced::core::{Clipboard, Event, Layout, Length, Rectangle, Shell, Size, Vector, Widget};
use cosmic::{Element, Renderer, Theme};

/// Reports when the cursor enters and leaves its content, along with the content's bounds so a popup can be
/// anchored to it. Unlike a mouse area, which only says that the cursor entered.
pub struct HoverArea<'a, Message> {
    content: Element<'a, Message>,
    on_enter: Box<dyn Fn(Rectangle) -> Message + 'a>,
    on_exit: Message,
}

pub fn hover_area<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    on_enter: impl Fn(Rectangle) -> Message + 'a,
    on_exit: Message,
) -> HoverArea<'a, Message> {
    HoverArea {
        content: content.into(),
        on_enter: Box::new(on_enter),
        on_exit,
    }
}

#[derive(Default)]
struct State {
    is_hovered: bool,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for HoverArea<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<State>();
        let is_hovered = match event {
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorEntered) => {
                cursor.is_over(layout.bounds())
            }
            Event::Mouse(mouse::Event::CursorLeft) => false,
            _ => return,
        };

        if is_hovered != state.is_hovered {
            state.is_hovered = is_hovered;
            shell.publish(if is_hovered {
                (self.on_enter)(layout.bounds())
            } else {
                self.on_exit.clone()
            });
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer, viewport, translation)
    }
}

impl<'a, Message: Clone + 'a> From<HoverArea<'a, Message>> for Element<'a, Message> {
    fn from(area: HoverArea<'a, Message>) -> Self {
        Element::new(area)
    }
}
