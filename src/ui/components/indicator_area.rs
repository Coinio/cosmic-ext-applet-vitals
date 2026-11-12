use cosmic::iced::core::layout;
use cosmic::iced::core::mouse;
use cosmic::iced::core::overlay;
use cosmic::iced::core::renderer::{self, Renderer as _};
use cosmic::iced::core::touch;
use cosmic::iced::core::widget::{tree, Operation, Tree};
use cosmic::iced::core::{
    Border, Clipboard, Color, Event, Layout, Length, Rectangle, Shell, Size, Vector, Widget,
};
use cosmic::{Element, Renderer, Theme};

/// One indicator in the applet, which is highlighted while the cursor is over it and reports when it's clicked.
///
/// It can also report when the cursor enters and leaves it, along with its bounds so a popup can be anchored to it.
/// Unlike a mouse area, which only says that the cursor entered.
pub struct IndicatorArea<'a, Message> {
    content: Element<'a, Message>,
    on_press: Message,
    on_hover: Option<OnHover<'a, Message>>,
}

struct OnHover<'a, Message> {
    on_enter: Box<dyn Fn(Rectangle) -> Message + 'a>,
    on_exit: Message,
}

pub fn indicator_area<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> IndicatorArea<'a, Message> {
    IndicatorArea {
        content: content.into(),
        on_press,
        on_hover: None,
    }
}

impl<'a, Message> IndicatorArea<'a, Message> {
    /// Report when the cursor enters, with the indicator's bounds, and leaves.
    pub fn on_hover(mut self, on_enter: impl Fn(Rectangle) -> Message + 'a, on_exit: Message) -> Self {
        self.on_hover = Some(OnHover {
            on_enter: Box::new(on_enter),
            on_exit,
        });
        self
    }
}

#[derive(Default)]
struct State {
    is_hovered: bool,
    /// Pressed, and not yet released. The press counts as a click if it's released over the indicator.
    is_pressed: bool,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for IndicatorArea<'_, Message> {
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
        if shell.is_event_captured() {
            return;
        }

        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorEntered) => {
                self.set_hovered(state, cursor.is_over(bounds), bounds, shell);
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                self.set_hovered(state, false, bounds, shell);
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(bounds) {
                    state.is_pressed = true;
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if state.is_pressed {
                    state.is_pressed = false;
                    if cursor.is_over(bounds) {
                        shell.publish(self.on_press.clone());
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                if state.is_pressed {
                    state.is_pressed = false;
                    shell.request_redraw();
                }
            }
            _ => {}
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

    /// While hovered, the indicator gets the background the applet button has when it's hovered, and while pressed,
    /// when it's pressed. The corners match the button's, so at either end of the applet the highlight follows its
    /// rounded edge.
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
        let state = tree.state.downcast_ref::<State>();
        let cosmic = theme.cosmic();
        let background = if state.is_pressed && cursor.is_over(layout.bounds()) {
            Some(cosmic.text_button.pressed)
        } else if state.is_hovered {
            Some(cosmic.text_button.hover)
        } else {
            None
        };
        if let Some(background) = background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: Border {
                        radius: cosmic.corner_radii.radius_xl.into(),
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                Color::from(background),
            );
        }

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

impl<Message: Clone> IndicatorArea<'_, Message> {
    fn set_hovered(&self, state: &mut State, is_hovered: bool, bounds: Rectangle, shell: &mut Shell<'_, Message>) {
        if is_hovered == state.is_hovered {
            return;
        }
        state.is_hovered = is_hovered;
        shell.request_redraw();
        if let Some(on_hover) = &self.on_hover {
            shell.publish(if is_hovered {
                (on_hover.on_enter)(bounds)
            } else {
                on_hover.on_exit.clone()
            });
        }
    }
}

impl<'a, Message: Clone + 'a> From<IndicatorArea<'a, Message>> for Element<'a, Message> {
    fn from(area: IndicatorArea<'a, Message>) -> Self {
        Element::new(area)
    }
}
